use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use rust_tdlib::types::MessageContent;

/// Filter by message type
#[derive(Debug, Clone)]
pub enum MessageType {
    Text,
    Video,
    Photo,
    Animation,
    Document,
    AnyFile,
}

impl Filter for MessageType {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        let message_content = data.input.message().content();

        match self {
            MessageType::Text => match message_content {
                MessageContent::MessageText(_) => Ok(()),
                _ => Err(()),
            },
            MessageType::Video => match message_content {
                MessageContent::MessageVideo(_) => Ok(()),
                _ => Err(()),
            },
            MessageType::Photo => match message_content {
                MessageContent::MessagePhoto(_) => Ok(()),
                _ => Err(()),
            },
            MessageType::Document => match message_content {
                MessageContent::MessageDocument(_) => Ok(()),
                _ => Err(()),
            },
            MessageType::Animation => match message_content {
                MessageContent::MessageAnimation(_) => Ok(()),
                MessageContent::MessageDocument(doc)
                    if doc.document().mime_type().eq("image/gif") =>
                {
                    Ok(())
                }
                _ => Err(()),
            },
            MessageType::AnyFile => {
                if matches!(
                    message_content,
                    MessageContent::MessageVideo(_)
                        | MessageContent::MessagePhoto(_)
                        | MessageContent::MessageAnimation(_)
                        | MessageContent::MessageDocument(_)
                ) {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    #[tokio::test]
    async fn test_file() {
        let success_data = vec![
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Video(None, 0, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Document(None, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Photo(None, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Animation(None, 0, 0),
                false,
            )),
        ];
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));

        let filter = FilterType::from(FilterConf::AnyFile);

        for datum in &success_data {
            assert_eq!(Ok(()), filter.filter(datum).await);
        }

        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }

    #[tokio::test]
    async fn test_animation() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Animation(None, 0, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Animation);

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }

    #[tokio::test]
    async fn test_video() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Video(None, 0, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Video);

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }

    #[tokio::test]
    async fn test_document() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Document(None, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Document);

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }

    #[tokio::test]
    async fn test_photo() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Photo(None, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Photo);

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }

    #[tokio::test]
    async fn test_text() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Photo(None, 0),
            false,
        ));
        let filter = FilterType::from(FilterConf::Text);

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }
}
