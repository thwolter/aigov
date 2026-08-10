use clap::{Args, ValueEnum};

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
