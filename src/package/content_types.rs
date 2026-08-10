use super::insert_before_closing_tag;
use crate::error;

pub(crate) const PART_NAME: &str = "[Content_Types].xml";

pub(crate) fn ensure_override(
    xml: &[u8],
    part_name: &str,
    content_type: &str,
) -> error::Result<Vec<u8>> {
    insert_before_closing_tag(
        xml,
        "</Types>",
        &format!("<Override PartName=\"{part_name}\" ContentType=\"{content_type}\"/>"),
        &format!("PartName=\"{part_name}\""),
    )
}
