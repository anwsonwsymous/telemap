use crate::config::PipeConf;
use crate::processing::data::DataHub;
use crate::processing::transform;
use rust_tdlib::types::{
    FormattedText, InputMessageAnimation, InputMessageContent, InputMessagePhoto, InputMessageText,
    InputMessageVideo,
};

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
}

/// Forward trait calls
impl Pipe for PipeType {
    fn handle(&self, data: &mut DataHub) {
        match self {
            Self::Transform(p) => p.handle(data),
            Self::StaticText(p) => p.handle(data),
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

#[derive(Debug, Default, Clone)]
pub struct StaticText {
    formatted_text: FormattedText,
}

/// Sets static text on send message. On media content this will set "caption", otherwise "text"
impl StaticText {
    pub fn builder() -> StaticTextBuilder {
        let inner = StaticText::default();
        StaticTextBuilder { inner }
    }
}

impl Pipe for StaticText {
    fn handle(&self, data: &mut DataHub) {
        match &data.output {
            Some(InputMessageContent::InputMessageVideo(m)) => {
                data.output = Some(InputMessageContent::InputMessageVideo(
                    InputMessageVideo::builder()
                        .video(m.video())
                        .caption(&self.formatted_text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessagePhoto(m)) => {
                data.output = Some(InputMessageContent::InputMessagePhoto(
                    InputMessagePhoto::builder()
                        .photo(m.photo())
                        .caption(&self.formatted_text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessageAnimation(m)) => {
                data.output = Some(InputMessageContent::InputMessageAnimation(
                    InputMessageAnimation::builder()
                        .animation(m.animation())
                        .caption(&self.formatted_text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessageText(_)) => {
                data.output = Some(InputMessageContent::InputMessageText(
                    InputMessageText::builder()
                        .text(&self.formatted_text)
                        .build(),
                ))
            }
            _ => (),
        };
    }
}

/// Builder struct for StaticText pipe
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

#[cfg(test)]
mod tests {
    use crate::config::PipeConf;
    use crate::processing::data::DataHub;
    use crate::processing::pipe::{Pipe, PipeType};
    use crate::processing::test_helpers::{
        formatted_text_example, message_example, sender_user_example, MessageMock,
    };
    use rust_tdlib::types::InputMessageContent;

    fn transformed_data_example() -> DataHub {
        let mut data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));

        let pipe = PipeType::from(PipeConf::Transform);
        pipe.handle(&mut data);
        data
    }

    #[test]
    fn test_transform() {
        let data = transformed_data_example();

        assert!(data.output.is_some());
    }

    #[test]
    fn test_static_text() {
        let mut data = transformed_data_example();
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
}
