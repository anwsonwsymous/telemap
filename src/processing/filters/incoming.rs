use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};

/// Filter only incoming messages
#[derive(Debug, Clone)]
pub struct Incoming;

impl Filter for Incoming {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        if !data.input.message().is_outgoing() {
            Ok(())
        } else {
            Err(())
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
    async fn test_incoming() {
        let outgoing_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            true,
        ));
        let incoming_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));

        let filter = FilterType::from(FilterConf::Incoming);

        assert_eq!(Ok(()), filter.filter(&incoming_data).await);
        assert_eq!(Err(()), filter.filter(&outgoing_data).await);
    }
}
