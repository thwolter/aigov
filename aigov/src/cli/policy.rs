use crate::cli::{Cli, FileCommand, print_success, print_warning};
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::policy::AiPolicy;
use clap::{Args, CommandFactory, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    command: Option<PolicyCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
enum PolicyPreset {
    /// Allow training and do not require review or attribution.
    Permissive,

    /// Require human review but not attribution.
    Standard,

    /// Disallow training and require review and attribution.
    Restricted,
}

impl PolicyPreset {
    pub fn policy(&self) -> AiPolicy {
        match self {
            Self::Permissive => AiPolicy {
                id: "permissive".into(),
                human_review_required: false,
                training_allowed: true,
                attribution_required: false,
                owner: None,
            },
            Self::Standard => AiPolicy {
                id: "standard".into(),
                human_review_required: true,
                training_allowed: false,
                attribution_required: false,
                owner: None,
            },
            Self::Restricted => AiPolicy {
                id: "restricted".into(),
                human_review_required: true,
                training_allowed: false,
                attribution_required: true,
                owner: None,
            },
        }
    }
}

#[derive(Subcommand)]
enum PolicyCommand {
    /// Inject policy from a preset
    Inject(InjectArgs),

    /// Update policy with a preset
    Update(UpdateArgs),

    /// Remove a policy if one exists
    Remove,

    /// Validate policy
    Validate,
}

#[derive(Args)]
struct InjectArgs {
    #[arg(short, long, help = "Output file path")]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Policy preset to apply")]
    preset: Option<PolicyPreset>,
}

#[derive(Args)]
struct UpdateArgs {
    #[arg(short, long, help = "Policy preset to apply")]
    preset: Option<PolicyPreset>,
}

pub fn run(command: &FileCommand<PolicyArgs>) -> error::Result<()> {
    let Some(policy_command) = command.args.command.as_ref() else {
        Cli::command()
            .find_subcommand_mut("policy")
            .expect("policy subcommand is defined")
            .print_help()?;
        return Ok(());
    };
    let mut document = Document::from_file(&command.input)?;
    let output = output_path(command.input.as_path(), policy_command);

    {
        let Some(policy_document) = document.as_ai_policy_document() else {
            return Err(error::OfficeError::UnsupportedFileType(
                "document format does not support AI policies".into(),
            ));
        };
        match policy_command {
            PolicyCommand::Inject(apply_args) => {
                let policy = apply_args
                    .preset
                    .as_ref()
                    .unwrap_or(&PolicyPreset::Standard)
                    .policy();
                policy_document.inject_policy(&policy)?;
                print_success("Policy injected")
            }
            PolicyCommand::Update(apply_args) => {
                let policy = apply_args
                    .preset
                    .as_ref()
                    .unwrap_or(&PolicyPreset::Standard)
                    .policy();
                policy_document.update_policy(&policy)?;
                print_success("Policy updated")
            }
            PolicyCommand::Remove => match policy_document.remove_policy()? {
                true => print_success("Policy removed"),
                false => print_warning("No policy to remove"),
            },
            PolicyCommand::Validate => policy_document.validate_policy()?,
        }
    }
    document.save(output)?;
    Ok(())
}

fn output_path<'a>(filepath: &'a Path, command: &'a PolicyCommand) -> &'a Path {
    match command {
        PolicyCommand::Inject(args) => args.output.as_deref().unwrap_or(filepath),
        _ => filepath,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{InjectArgs, PolicyCommand, PolicyPreset, output_path};

    #[test]
    fn presets_map_to_their_expected_policies() {
        let permissive = PolicyPreset::Permissive.policy();
        let standard = PolicyPreset::Standard.policy();
        let restricted = PolicyPreset::Restricted.policy();

        assert!(permissive.training_allowed);
        assert!(standard.human_review_required);
        assert!(!standard.attribution_required);
        assert!(restricted.attribution_required);
        assert!(!restricted.training_allowed);
    }

    #[test]
    fn inject_uses_requested_output_path() {
        let command = PolicyCommand::Inject(InjectArgs {
            output: Some(PathBuf::from("protected.docx")),
            preset: None,
        });

        assert_eq!(
            output_path(Path::new("source.docx"), &command),
            Path::new("protected.docx")
        );
    }
}
