use crate::config::PipeConf;
use crate::processing::data::DataHub;
use crate::processing::pipe::{Pipe, PipeType};
use rust_tdlib::types::{
    Animation, Document, File, FormattedText, LocalFile, Message, MessageAnimation, MessageContent,
    MessageDocument, MessagePhoto, MessageSender, MessageSenderUser, MessageText, MessageVideo,
    Photo, PhotoSize, RemoteFile, UpdateNewMessage, Video,
};

/// Mock message with all types of message contents.
pub(crate) enum MessageMock {
    Photo(Option<String>, i32),
    Document(Option<String>, i32),
    Video(Option<String>, i32, i32),
    Animation(Option<String>, i32, i32),
    Text(Option<String>),
}

/// Conversion from Mock to telegram's MessageContent
impl From<MessageMock> for MessageContent {
    fn from(mock: MessageMock) -> Self {
        match mock {
            MessageMock::Photo(text, filesize) => MessageContent::MessagePhoto(
                MessagePhoto::builder()
                    .photo(
                        Photo::builder()
                            .sizes(vec![PhotoSize::builder()
                                .photo(file_example(filesize))
                                .build()])
                            .build(),
                    )
                    .caption(formatted_text_example(text))
                    .build(),
            ),
            MessageMock::Video(text, duration, filesize) => MessageContent::MessageVideo(
                MessageVideo::builder()
                    .video(
                        Video::builder()
                            .video(file_example(filesize))
                            .duration(duration)
                            .build(),
                    )
                    .caption(formatted_text_example(text))
                    .build(),
            ),
            MessageMock::Animation(text, duration, filesize) => MessageContent::MessageAnimation(
                MessageAnimation::builder()
                    .animation(
                        Animation::builder()
                            .animation(file_example(filesize))
                            .duration(duration)
                            .build(),
                    )
                    .caption(formatted_text_example(text))
                    .build(),
            ),
            MessageMock::Document(text, filesize) => MessageContent::MessageDocument(
                MessageDocument::builder()
                    .document(Document::builder().document(file_example(filesize)).build())
                    .caption(formatted_text_example(text))
                    .build(),
            ),
            MessageMock::Text(text) => MessageContent::MessageText(Box::new(
                MessageText::builder()
                    .text(formatted_text_example(text))
                    .build(),
            )),
        }
    }
}

pub(crate) fn sender_user_example() -> MessageSender {
    MessageSender::User(MessageSenderUser::builder().user_id(1).build())
}

pub(crate) fn file_example(size: i32) -> File {
    File::builder()
        .size(size)
        .expected_size(size)
        .remote(RemoteFile::builder().build())
        .local(LocalFile::builder().build())
        .build()
}

pub(crate) fn formatted_text_example(text: Option<String>) -> FormattedText {
    FormattedText::builder()
        .text(text.unwrap_or("example".to_string()))
        .build()
}

pub(crate) fn message_example(
    sender: MessageSender,
    content: MessageMock,
    outgoing: bool,
) -> UpdateNewMessage {
    UpdateNewMessage::builder()
        .message(
            Message::builder()
                .id(1)
                .chat_id(1)
                .is_outgoing(outgoing)
                .sender_id(sender)
                .content(MessageContent::from(content))
                .build(),
        )
        .build()
}

pub(crate) fn transformed_data_example(message: Option<String>) -> DataHub {
    let mut data = DataHub::new(message_example(
        sender_user_example(),
        MessageMock::Text(message),
        false,
    ));

    let pipe = PipeType::from(PipeConf::Transform);
    pipe.handle(&mut data);
    data
}
