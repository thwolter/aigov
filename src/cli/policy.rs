use crate::cli::{Cli, print_success, print_warning};
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::policy::AiPolicy;
use clap::{Args, CommandFactory, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Args)]
pub struct PolicyArgs {
    #[arg(short, long, help = "Output file path")]
    output: Option<PathBuf>,

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
    Inject(ApplyPolicyArgs),

    /// Update policy with a preset
    Update(ApplyPolicyArgs),

    /// Remove a policy if one exists
    Remove,

    /// Validate policy
    Validate,
}

#[derive(Args)]
struct ApplyPolicyArgs {
    #[arg(short, long, help = "Policy preset to apply")]
    preset: Option<PolicyPreset>,
}

pub fn run(filepath: &PathBuf, args: &PolicyArgs) -> error::Result<()> {
    let Some(command) = args.command.as_ref() else {
        Cli::command()
            .find_subcommand_mut("policy")
            .expect("policy subcommand is defined")
            .print_help()?;
        return Ok(());
    };
    let mut document = Document::from_file(filepath)?;
    let output = args.output.as_deref().unwrap_or(filepath);

    {
        let Some(policy_document) = document.as_ai_policy_document() else {
            return Err(error::OfficeError::UnsupportedFileType(
                "document format does not support AI policies".into(),
            ));
        };
        match command {
            PolicyCommand::Inject(apply_args) => {
                policy_document.inject_policy(&resolve_policy(apply_args))?;
                print_success("Policy injected")
            }
            PolicyCommand::Update(apply_args) => {
                policy_document.update_policy(&resolve_policy(apply_args))?;
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

fn resolve_policy(args: &ApplyPolicyArgs) -> AiPolicy {
    args.preset
        .as_ref()
        .unwrap_or(&PolicyPreset::Standard)
        .policy()
}

#[cfg(test)]
mod tests {
    use super::PolicyPreset;

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
}
