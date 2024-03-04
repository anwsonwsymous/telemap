use rust_tdlib::types::{
    FormattedText, InputMessageAnimation, InputMessageContent, InputMessagePhoto, InputMessageText,
    InputMessageVideo, UpdateNewMessage,
};

#[derive(Debug)]
/// Main struct which contains input message and output builder.
pub struct DataHub {
    /// Telegram Update type
    pub input: UpdateNewMessage,
    /// Optional message content. This is used in SendMessage::builder().input_message_content(here)
    pub output: Option<InputMessageContent>,
}

impl DataHub {
    pub fn new(input: UpdateNewMessage) -> Self {
        DataHub {
            input,
            output: None,
        }
    }

    pub fn set_output_text(&mut self, text: FormattedText) {
        match &self.output {
            Some(InputMessageContent::InputMessageVideo(m)) => {
                self.output = Some(InputMessageContent::InputMessageVideo(
                    InputMessageVideo::builder()
                        .video(m.video())
                        .caption(text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessagePhoto(m)) => {
                self.output = Some(InputMessageContent::InputMessagePhoto(
                    InputMessagePhoto::builder()
                        .photo(m.photo())
                        .caption(text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessageAnimation(m)) => {
                self.output = Some(InputMessageContent::InputMessageAnimation(
                    InputMessageAnimation::builder()
                        .animation(m.animation())
                        .caption(text)
                        .build(),
                ));
            }
            Some(InputMessageContent::InputMessageText(_)) => {
                self.output = Some(InputMessageContent::InputMessageText(
                    InputMessageText::builder().text(text).build(),
                ))
            }
            _ => (),
        };
    }
}
