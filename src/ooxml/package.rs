use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};
use std::io::Seek;
use zip::{
    write::SimpleFileOptions,
    ZipArchive,
    ZipWriter,
};

use crate::{
    office::PartName,
    error::{Result, OfficeError},
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
    where R: Read + Seek
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

            parts.insert(
                PartName::new(entry.name()),
                content,
            );
        }
        Ok(parts)
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn parts(&self) -> Vec<PartName> {
        let mut parts: Vec<_> =
            self.parts.keys().cloned().collect();

        parts.sort_by(|left, right| {
            left.as_str().cmp(right.as_str())
        });

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

        parts.sort_by(|(left, _), (right, _)| {
            left.as_str().cmp(right.as_str())
        });

        for (part_name, content) in parts {
            archive.start_file(part_name.as_str(), options)?;
            archive.write_all(content)?;
        }

        archive.finish()?;

        Ok(())
    }
}