use clap::{Args, Subcommand};

#[derive(Args)]
pub struct MetadataArgs {
    #[arg(short, long)]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Option<MetadataCommand>,
}

#[derive(Subcommand)]
pub enum MetadataCommand {
    #[command(override_usage = "aigov <FILEPATH> metadata set [OPTIONS]")]
    Set(SetArgs),
}

#[derive(Args)]
#[command(
    about = "Update editable document metadata; application-generated statistics remain read-only"
)]
pub struct SetArgs {
    #[arg(long, short)]
    pub title: Option<String>,

    #[arg(long, short)]
    pub description: Option<String>,

    #[arg(long, short)]
    pub author: Option<String>,

    #[arg(long, short)]
    pub keywords: Option<String>,

    #[arg(long, short)]
    pub creator: Option<String>,

    #[arg(long, short)]
    pub subject: Option<String>,

    #[arg(long)]
    pub category: Option<String>,

    #[arg(long = "content-status")]
    pub content_status: Option<String>,

    #[arg(long = "content-type")]
    pub content_type: Option<String>,

    #[arg(long)]
    pub language: Option<String>,

    #[arg(long)]
    pub identifier: Option<String>,

    #[arg(long, short)]
    pub version: Option<String>,
}

impl SetArgs {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.author.is_none()
            && self.keywords.is_none()
            && self.creator.is_none()
            && self.subject.is_none()
            && self.category.is_none()
            && self.content_status.is_none()
            && self.content_type.is_none()
            && self.language.is_none()
            && self.identifier.is_none()
            && self.version.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::SetArgs;

    #[test]
    fn detects_an_empty_update() {
        assert!(
            SetArgs {
                title: None,
                description: None,
                author: None,
                keywords: None,
                creator: None,
                subject: None,
                category: None,
                content_status: None,
                content_type: None,
                language: None,
                identifier: None,
                version: None,
            }
                .is_empty()
        );
    }
}
