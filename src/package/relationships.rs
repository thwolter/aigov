use super::insert_before_closing_tag;
use crate::error;

pub(crate) const PART_NAME: &str = "_rels/.rels";

pub(crate) fn ensure_relationship(
    xml: &[u8],
    id: &str,
    relationship_type: &str,
    target: &str,
) -> error::Result<Vec<u8>> {
    insert_before_closing_tag(
        xml,
        "</Relationships>",
        &format!("<Relationship Id=\"{id}\" Type=\"{relationship_type}\" Target=\"{target}\"/>"),
        &format!("Type=\"{relationship_type}\""),
    )
}
