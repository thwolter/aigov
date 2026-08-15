use crate::error::Result;

pub const PART_NAME: &str = "_rels/.rels";

pub fn ensure_relationship(
    xml: &[u8],
    id: &str,
    relationship_type: &str,
    target: &str,
) -> Result<Vec<u8>> {
    super::insert_before_closing_tag(
        xml,
        "</Relationships>",
        &format!("<Relationship Id=\"{id}\" Type=\"{relationship_type}\" Target=\"{target}\"/>"),
        &format!("Type=\"{relationship_type}\""),
    )
}
