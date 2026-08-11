use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::policy::AiPolicy;
use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum InjectPolicy {
    /// Apply the least restrictive predefined policy.
    Weak,

    /// Apply the most restrictive predefined policy.
    Strong,
}

#[derive(Args)]
pub struct PolicyArgs {
    #[arg(
        short,
        long,
        value_enum,
        value_name = "STRENGTH",
        help = "Inject a predefined AI policy"
    )]
    pub inject: Option<InjectPolicy>,
}

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
