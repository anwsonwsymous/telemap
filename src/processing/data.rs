use rust_tdlib::types::{InputMessageContent, UpdateNewMessage};

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
}
