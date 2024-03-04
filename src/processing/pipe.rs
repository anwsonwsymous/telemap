use crate::config::PipeConf;
use crate::processing::data::DataHub;
use crate::processing::helpers::find_output_message_text;
use crate::processing::transform;
use rust_tdlib::types::FormattedText;

/// Pipe trait handles received messages and makes output builder (SendMessageBuilder)
pub trait Pipe {
    fn handle(&self, data: &mut DataHub);
}

#[derive(Debug, Clone)]
/// Available pipe types
pub enum PipeType {
    /// Just transform received message into send message type
    Transform(Transform),
    /// Sets static text on send message. On media content this will set "caption", otherwise "text"
    StaticText(StaticText),
    /// Search and replace text on send message
    Replace(Replace),
}

/// Forward trait calls
impl Pipe for PipeType {
    fn handle(&self, data: &mut DataHub) {
        match self {
            Self::Transform(p) => p.handle(data),
            Self::StaticText(p) => p.handle(data),
            Self::Replace(p) => p.handle(data),
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

            PipeConf::Replace { search, replace } => {
                PipeType::Replace(Replace::builder().search(search).replace(replace).build())
            }
        }
    }
}

/// Transform received message into send message.
#[derive(Debug, Clone)]
pub struct Transform;

impl Pipe for Transform {
    fn handle(&self, data: &mut DataHub) {
        // All type of messages (text, video, photo, animation, etc...)
        if let Ok(new_message) = transform(data.input.message()) {
            data.output = Some(new_message);
        }
    }
}

/// Sets static text on send message. On media content this will set "caption", otherwise "text"
#[derive(Debug, Default, Clone)]
pub struct StaticText {
    formatted_text: FormattedText,
}

impl StaticText {
    pub fn builder() -> StaticTextBuilder {
        let inner = StaticText::default();
        StaticTextBuilder { inner }
    }
}

impl Pipe for StaticText {
    fn handle(&self, data: &mut DataHub) {
        data.set_output_text(self.formatted_text.clone());
    }
}

pub struct StaticTextBuilder {
    inner: StaticText,
}

impl StaticTextBuilder {
    pub fn text(&mut self, text: FormattedText) -> &mut StaticTextBuilder {
        self.inner.formatted_text = text;
        self
    }

    pub fn build(&self) -> StaticText {
        self.inner.clone()
    }
}

/// Search and replace text on send message
#[derive(Debug, Default, Clone)]
pub struct Replace {
    search: String,
    replace: String,
}

impl Replace {
    pub fn builder() -> ReplaceBuilder {
        let inner = Replace::default();
        ReplaceBuilder { inner }
    }
}

impl Pipe for Replace {
    fn handle(&self, data: &mut DataHub) {
        if let Some(m) = &data.output {
            if let Some(formatted_text) = find_output_message_text(m) {
                let replaced_text = formatted_text.text().replace(&self.search, &self.replace);
                if formatted_text.text().eq(&replaced_text) {
                    return;
                }

                let new_formatted_text = FormattedText::builder()
                    .text(replaced_text)
                    .entities(formatted_text.entities().clone())
                    .build();

                data.set_output_text(new_formatted_text);
            }
        }
    }
}

pub struct ReplaceBuilder {
    inner: Replace,
}

impl ReplaceBuilder {
    pub fn search(&mut self, search: String) -> &mut ReplaceBuilder {
        self.inner.search = search;
        self
    }

    pub fn replace(&mut self, replace: String) -> &mut ReplaceBuilder {
        self.inner.replace = replace;
        self
    }

    pub fn build(&self) -> Replace {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::PipeConf;
    use crate::processing::data::DataHub;
    use crate::processing::pipe::{Pipe, PipeType};
    use crate::processing::test_helpers::{
        formatted_text_example, message_example, sender_user_example, MessageMock,
    };
    use rust_tdlib::types::InputMessageContent;

    fn transformed_data_example(message: Option<String>) -> DataHub {
        let mut data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(message),
            false,
        ));

        let pipe = PipeType::from(PipeConf::Transform);
        pipe.handle(&mut data);
        data
    }

    #[test]
    fn test_transform() {
        let data = transformed_data_example(None);

        assert!(data.output.is_some());
    }

    #[test]
    fn test_static_text() {
        let mut data = transformed_data_example(None);
        let success_formatted_text = formatted_text_example(Some("Test Text".to_string()));
        let pipe = PipeType::from(PipeConf::StaticText {
            formatted_text: success_formatted_text.clone(),
        });
        pipe.handle(&mut data);

        println!("{:?}", success_formatted_text);
        println!("{:?}", pipe);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(success_formatted_text.text(), data_text.text());
    }

    #[test]
    fn test_replace() {
        let mut data = transformed_data_example(Some("Search Search".to_string()));
        let success_text = formatted_text_example(Some("Replaced Replaced".to_string()));
        let pipe = PipeType::from(PipeConf::Replace {
            search: "Search".to_string(),
            replace: "Replaced".to_string(),
        });

        pipe.handle(&mut data);

        println!("{:?}", pipe);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(data_text.text(), success_text.text());
    }
}
