use crate::cli::policy::PolicyArgs;
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::policy::AiPolicy;
use std::path::PathBuf;

pub fn handle_policy(filepath: &PathBuf, _args: &PolicyArgs) -> error::Result<()> {
    let mut document = Document::from_file(filepath)?;

    {
        let Some(policy_document) = document.as_ai_policy_document() else {
            return Err(error::OfficeError::UnsupportedFileType(
                "document format does not support AI policies".into(),
            ));
        };
        let policy = AiPolicy {
            human_review_required: true,
            ..AiPolicy::default()
        };
        policy_document.inject_policy(&policy)?;
    }
    document.save(filepath)?;
    Ok(())
}
