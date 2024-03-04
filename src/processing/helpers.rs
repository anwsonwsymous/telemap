use rust_tdlib::types::{
    File, FormattedText, InputFile, InputFileId, InputMessageAnimation, InputMessageContent,
    InputMessageDocument, InputMessagePhoto, InputMessageText, InputMessageVideo, Message,
    MessageContent,
};

/// Find file in message content (Video, Animation, Document, Photo).
/// For photo's this will return first photo size
pub(crate) fn find_input_message_file(message: &Message) -> Option<&File> {
    match message.content() {
        MessageContent::MessageVideo(m) => Some(m.video().video()),
        MessageContent::MessagePhoto(m) => Some(m.photo().sizes().first()?.photo()),
        MessageContent::MessageAnimation(m) => Some(m.animation().animation()),
        MessageContent::MessageDocument(m) => Some(m.document().document()),
        _ => None,
    }
}

/// Find Text message in message content, for media's return caption
pub(crate) fn find_input_message_text(message: &Message) -> Option<&String> {
    match message.content() {
        MessageContent::MessageText(m) => Some(m.text().text()),
        MessageContent::MessagePhoto(m) => Some(m.caption().text()),
        MessageContent::MessageAnimation(m) => Some(m.caption().text()),
        MessageContent::MessageVideo(m) => Some(m.caption().text()),
        _ => None,
    }
}

/// Find message duration from video/animation medias.
pub(crate) fn find_input_message_duration(message: &Message) -> Option<i32> {
    match message.content() {
        MessageContent::MessageVideo(m) => Some(m.video().duration()),
        MessageContent::MessageAnimation(m) => Some(m.animation().duration()),
        _ => None,
    }
}

/// Find Text message from InputMessageContent
pub(crate) fn find_output_message_text(message: &InputMessageContent) -> Option<&FormattedText> {
    match message {
        InputMessageContent::InputMessageText(m) => Some(m.text()),
        InputMessageContent::InputMessageVideo(m) => Some(m.caption()),
        InputMessageContent::InputMessagePhoto(m) => Some(m.caption()),
        InputMessageContent::InputMessageAnimation(m) => Some(m.caption()),
        _ => None,
    }
}

// TODO: Try with impl PartialOrd
/// Compare function with string operator
pub(crate) fn cmp<T: PartialOrd>(operator: &str, left: &T, right: &T) -> bool {
    match operator {
        "<" if left < right => true,
        ">" if left > right => true,
        "=" if left == right => true,
        ">=" if left >= right => true,
        "<=" if left <= right => true,
        _ => false,
    }
}

/// Transform input message into output message
pub(crate) fn transform(input: &Message) -> Result<InputMessageContent, ()> {
    match input.content() {
        MessageContent::MessageText(received_message) => Ok(InputMessageContent::InputMessageText(
            InputMessageText::builder()
                .text(received_message.text())
                .build(),
        )),

        MessageContent::MessageVideo(received_message) => {
            let video = InputFile::Id(
                InputFileId::builder()
                    .id(received_message.video().video().id())
                    .build(),
            );

            Ok(InputMessageContent::InputMessageVideo(
                InputMessageVideo::builder()
                    .video(video)
                    .caption(received_message.caption())
                    .build(),
            ))
        }

        MessageContent::MessageAnimation(received_message) => {
            let animation = InputFile::Id(
                InputFileId::builder()
                    .id(received_message.animation().animation().id())
                    .build(),
            );

            Ok(InputMessageContent::InputMessageAnimation(
                InputMessageAnimation::builder()
                    .animation(animation)
                    .caption(received_message.caption())
                    .build(),
            ))
        }

        MessageContent::MessageDocument(received_message) => {
            let doc = InputFile::Id(
                InputFileId::builder()
                    .id(received_message.document().document().id())
                    .build(),
            );

            Ok(InputMessageContent::InputMessageDocument(
                InputMessageDocument::builder()
                    .document(doc)
                    .caption(received_message.caption())
                    .build(),
            ))
        }

        MessageContent::MessagePhoto(received_message) => {
            let photo_size = received_message.photo().sizes().first().unwrap();
            let photo = InputFile::Id(InputFileId::builder().id(photo_size.photo().id()).build());

            Ok(InputMessageContent::InputMessagePhoto(
                InputMessagePhoto::builder()
                    .photo(photo)
                    .caption(received_message.caption())
                    .build(),
            ))
        }

        _ => Err(()),
    }
}
