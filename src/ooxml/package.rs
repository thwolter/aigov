use std::io::Seek;
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::{content_types, relationships};
use crate::{
    error::{OfficeError, Result},
    office::PartName,
};

#[derive(Debug, Clone)]
pub struct OoxmlPackage {
    source_path: Option<PathBuf>,
    parts: HashMap<PartName, Vec<u8>>,
}

impl OoxmlPackage {
    /// Creates a new package from a file path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let parts = Self::collect_parts(file)?;

        Ok(Self {
            source_path: Some(path.to_path_buf()),
            parts,
        })
    }

    /// Creates a new package from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);
        let parts = Self::collect_parts(cursor)?;

        Ok(Self {
            source_path: None,
            parts,
        })
    }

    fn collect_parts<R>(reader: R) -> Result<HashMap<PartName, Vec<u8>>>
    where
        R: Read + Seek,
    {
        let mut archive = ZipArchive::new(reader)?;
        let mut parts = HashMap::new();

        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;

            if entry.is_dir() {
                continue;
            }

            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;

            parts.insert(PartName::new(entry.name()), content);
        }
        Ok(parts)
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn parts(&self) -> Vec<PartName> {
        let mut parts: Vec<_> = self.parts.keys().cloned().collect();

        parts.sort_by(|left, right| left.as_str().cmp(right.as_str()));

        parts
    }

    pub fn contains_part(&self, part: &PartName) -> bool {
        self.parts.contains_key(part)
    }

    pub fn read_part(&self, part: &PartName) -> Result<&[u8]> {
        self.parts
            .get(part)
            .map(Vec::as_slice)
            .ok_or_else(|| OfficeError::PartNotFound(part.clone()))
    }

    pub(crate) fn ensure_content_type_override(
        &mut self,
        part_name: &str,
        content_type: &str,
    ) -> Result<()> {
        let part = PartName::from(content_types::PART_NAME);
        let xml = self.read_part(&part)?.to_vec();
        let updated = content_types::ensure_override(&xml, part_name, content_type)?;

        self.write_part(part, updated);
        Ok(())
    }

    pub(crate) fn ensure_root_relationship(
        &mut self,
        id: &str,
        relationship_type: &str,
        target: &str,
    ) -> Result<()> {
        let part = PartName::from(relationships::PART_NAME);
        let xml = self.read_part(&part)?.to_vec();
        let updated = relationships::ensure_relationship(
            &xml,
            id,
            relationship_type,
            target,
        )?;

        self.write_part(part, updated);
        Ok(())
    }

    /// Writes a part to the package.
    pub fn write_part(&mut self, part: PartName, content: Vec<u8>) {
        self.parts.insert(part, content);
    }

    pub fn remove_part(&mut self, part: &PartName) -> bool {
        self.parts.remove(part).is_some()
    }

    /// Saves the package to a file.
    pub fn save(&self, destination: &Path) -> Result<()> {
        let file = File::create(destination)?;
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        let mut parts: Vec<_> = self.parts.iter().collect();

        parts.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));

        for (part_name, content) in parts {
            archive.start_file(part_name.as_str(), options)?;
            archive.write_all(content)?;
        }

        archive.finish()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Cursor,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(&mut bytes);
        let options = SimpleFileOptions::default();

        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }

        writer.finish().unwrap();
        bytes.into_inner()
    }

    fn temporary_path() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!(
            "aigov-ooxml-package-{timestamp}-{}-{id}.zip",
            std::process::id()
        ))
    }

    #[test]
    fn from_bytes_loads_files_and_skips_directories() {
        let bytes = zip_bytes(&[
            ("word/", &[]),
            ("/word/document.xml", b"document"),
            ("word/styles.xml", b"styles"),
        ]);

        let package = OoxmlPackage::from_bytes(&bytes).unwrap();

        assert_eq!(package.source_path(), None);
        assert_eq!(
            package.parts(),
            vec![
                PartName::from("word/document.xml"),
                PartName::from("word/styles.xml")
            ]
        );
        assert_eq!(
            package
                .read_part(&PartName::from("word/document.xml"))
                .unwrap(),
            b"document"
        );
    }

    #[test]
    fn parts_are_sorted_and_part_names_are_normalized() {
        let bytes = zip_bytes(&[("z.xml", b"z"), ("/a.xml", b"a")]);
        let package = OoxmlPackage::from_bytes(&bytes).unwrap();

        assert_eq!(
            package.parts(),
            vec![PartName::from("a.xml"), PartName::from("z.xml")]
        );
        assert!(package.contains_part(&PartName::from("/a.xml")));
    }

    #[test]
    fn write_and_remove_part_update_the_package() {
        let mut package = OoxmlPackage::from_bytes(&zip_bytes(&[("old.xml", b"old")])).unwrap();
        let new_part = PartName::from("new.xml");

        package.write_part(new_part.clone(), b"new".to_vec());
        assert_eq!(package.read_part(&new_part).unwrap(), b"new");
        assert!(package.remove_part(&PartName::from("old.xml")));
        assert!(!package.remove_part(&PartName::from("missing.xml")));
        assert!(!package.contains_part(&PartName::from("old.xml")));
    }

    #[test]
    fn reading_a_missing_part_returns_a_descriptive_error() {
        let package = OoxmlPackage::from_bytes(&zip_bytes(&[])).unwrap();

        let error = package
            .read_part(&PartName::from("word/document.xml"))
            .unwrap_err();

        assert!(
            matches!(error, OfficeError::PartNotFound(ref part) if part == &PartName::from("word/document.xml"))
        );
        assert_eq!(
            error.to_string(),
            "Package part not found: word/document.xml"
        );
    }

    #[test]
    fn invalid_zip_bytes_are_rejected() {
        let error = OoxmlPackage::from_bytes(b"not a zip").unwrap_err();

        assert!(matches!(error, OfficeError::Zip(_)));
    }

    #[test]
    fn open_and_save_preserve_package_contents() {
        let source = temporary_path();
        let destination = temporary_path();
        let bytes = zip_bytes(&[
            ("word/document.xml", b"document"),
            ("docProps/core.xml", b"core"),
        ]);

        fs::write(&source, &bytes).unwrap();
        let package = OoxmlPackage::open(&source).unwrap();
        assert_eq!(package.source_path(), Some(source.as_path()));

        package.save(&destination).unwrap();
        let saved = OoxmlPackage::open(&destination).unwrap();
        assert_eq!(saved.parts(), package.parts());
        assert_eq!(
            saved
                .read_part(&PartName::from("word/document.xml"))
                .unwrap(),
            b"document"
        );
        assert_eq!(
            saved
                .read_part(&PartName::from("docProps/core.xml"))
                .unwrap(),
            b"core"
        );

        fs::remove_file(source).unwrap();
        fs::remove_file(destination).unwrap();
    }
}
