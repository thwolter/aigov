use crate::Result;
use super::xml::insert_before_closing_tag;

pub(crate) const PART_NAME: &str = "[Content_Types].xml";

pub(crate) fn ensure_override(
    xml: &[u8],
    part_name: &str,
    content_type: &str,
) -> Result<Vec<u8>> {
    insert_before_closing_tag(
        xml,
        "</Types>",
        &format!(
            "<Override PartName=\"{part_name}\" ContentType=\"{content_type}\"/>"
        ),
        &format!("PartName=\"{part_name}\""),
    )
}
