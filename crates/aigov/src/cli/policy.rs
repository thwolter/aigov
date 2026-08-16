use crate::cli::{DestinationArgs, FileCommand, print_success, print_warning};
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::policy::AiPolicy;
use clap::{Args, Subcommand, ValueEnum};

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct PolicyArgs {
    #[command(subcommand)]
    command: PolicyCommand,
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
    Remove(RemoveArgs),

    /// Validate policy
    Validate,
}

#[derive(Args)]
struct InjectArgs {
    #[arg(short, long, help = "Policy preset to apply")]
    preset: Option<PolicyPreset>,

    #[command(flatten)]
    destination: DestinationArgs,
}

#[derive(Args)]
struct UpdateArgs {
    #[arg(short, long, help = "Policy preset to apply")]
    preset: Option<PolicyPreset>,

    #[command(flatten)]
    destination: DestinationArgs,
}

#[derive(Args)]
struct RemoveArgs {
    #[command(flatten)]
    destination: DestinationArgs,
}

pub fn run(command: &FileCommand<PolicyArgs>) -> error::Result<()> {
    let mut document = Document::from_file(&command.input)?;

    {
        let Some(policy_document) = document.as_ai_policy_document() else {
            return Err(error::OfficeError::UnsupportedFileType(
                "document format does not support AI policies".into(),
            ));
        };
        match &command.args.command {
            PolicyCommand::Inject(apply_args) => {
                let policy = apply_args
                    .preset
                    .as_ref()
                    .unwrap_or(&PolicyPreset::Standard)
                    .policy();
                policy_document.inject_policy(&policy)?;
                document.save(apply_args.destination.output_or(&command.input))?;
                print_success("Policy injected")
            }
            PolicyCommand::Update(apply_args) => {
                let policy = apply_args
                    .preset
                    .as_ref()
                    .unwrap_or(&PolicyPreset::Standard)
                    .policy();
                policy_document.update_policy(&policy)?;
                document.save(apply_args.destination.output_or(&command.input))?;
                print_success("Policy updated")
            }
            PolicyCommand::Remove(apply_args) => {
                match policy_document.remove_policy()? {
                    true => print_success("Policy removed"),
                    false => print_warning("No policy to remove"),
                };
                document.save(apply_args.destination.output_or(&command.input))?;
            }
            PolicyCommand::Validate => policy_document.validate_policy()?,
        }
    }
    Ok(())
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
