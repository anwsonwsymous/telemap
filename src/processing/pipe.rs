use crate::config::PipeConf;
use crate::processing::data::DataHub;
use crate::processing::pipes::{
    Format, Replace, ReplaceRegexp, StaticPhoto, StaticText, Transform,
};

/// Pipe trait handles received messages and makes output builder (SendMessageBuilder)
pub trait Pipe {
    async fn handle(&self, data: &mut DataHub);
}

#[derive(Debug, Clone)]
/// Available pipe types
pub enum PipeType {
    /// Just transform received message into send message type
    Transform(Transform),
    /// Sets static text on send message. On media content this will set "caption", otherwise "text"
    StaticText(StaticText),
    /// Sets static photo on send message
    StaticPhoto(StaticPhoto),
    /// Search and replace text on send message
    Replace(Replace),
    /// Search and replace texts with regular expression
    ReplaceRegexp(ReplaceRegexp),
    /// Format send message by provided template
    #[cfg(feature = "templating")]
    Format(Format),
}

/// Forward trait calls
impl Pipe for PipeType {
    async fn handle(&self, data: &mut DataHub) {
        match self {
            Self::Transform(p) => p.handle(data).await,
            Self::StaticText(p) => p.handle(data).await,
            Self::StaticPhoto(p) => p.handle(data).await,
            Self::Replace(p) => p.handle(data).await,
            Self::ReplaceRegexp(p) => p.handle(data).await,
            #[cfg(feature = "templating")]
            Self::Format(p) => p.handle(data).await,
        }
    }
}

/// Build Pipe from config
impl From<PipeConf> for PipeType {
    fn from(pipe_conf: PipeConf) -> Self {
        match pipe_conf {
            PipeConf::Transform => PipeType::Transform(Transform),

            PipeConf::StaticText { formatted_text } => {
                PipeType::StaticText(StaticText::builder().text(formatted_text).build())
            }

            PipeConf::StaticPhoto { path } => {
                PipeType::StaticPhoto(StaticPhoto::builder().path(path).build())
            }

            PipeConf::Replace { search, replace } => {
                PipeType::Replace(Replace::builder().search(search).replace(replace).build())
            }

            #[cfg(feature = "templating")]
            PipeConf::Format { template } => {
                PipeType::Format(Format::builder().template(template).build())
            }

            PipeConf::ReplaceRegexp {
                search,
                replace,
                all,
            } => PipeType::ReplaceRegexp(
                ReplaceRegexp::builder()
                    .search(search)
                    .replace(replace)
                    .all(all)
                    .build(),
            ),
        }
    }
}
