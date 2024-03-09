use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::{cmp, find_input_message_text};

/// Filter by text length
#[derive(Debug, Default, Clone)]
pub struct TextLength {
    len: u16,
    /// Operator (<,>,=)
    op: String,
}

impl TextLength {
    pub fn builder() -> TextLengthBuilder {
        let inner = TextLength::default();
        TextLengthBuilder { inner }
    }
}

impl Filter for TextLength {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        let text = find_input_message_text(data.input.message());

        match cmp(&self.op, &text.ok_or(())?.len(), &(self.len as usize)) {
            true => Ok(()),
            false => Err(()),
        }
    }
}

pub struct TextLengthBuilder {
    inner: TextLength,
}

impl TextLengthBuilder {
    pub fn length(&mut self, len: u16) -> &mut TextLengthBuilder {
        self.inner.len = len;
        self
    }

    pub fn operator(&mut self, op: String) -> &mut TextLengthBuilder {
        self.inner.op = op;
        self
    }

    pub fn build(&self) -> TextLength {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    #[tokio::test]
    async fn test_text_length() {
        // 19 symbols
        let long_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("loong loong message".to_string())),
            false,
        ));
        // 5 symbols
        let short_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("short".to_string())),
            false,
        ));

        let less_filter = FilterType::from(FilterConf::TextLength {
            len: 10,
            op: "<".to_string(),
        });
        let eq_filter = FilterType::from(FilterConf::TextLength {
            len: 5,
            op: "=".to_string(),
        });
        let greater_filter = FilterType::from(FilterConf::TextLength {
            len: 6,
            op: ">".to_string(),
        });

        // Shorter than 10 symbols
        assert_eq!(Ok(()), less_filter.filter(&short_message_data).await);
        assert_eq!(Err(()), less_filter.filter(&long_message_data).await);

        // Equals to 5 symbols
        assert_eq!(Ok(()), eq_filter.filter(&short_message_data).await);
        assert_eq!(Err(()), eq_filter.filter(&long_message_data).await);

        // Greater than 5 symbols
        assert_eq!(Ok(()), greater_filter.filter(&long_message_data).await);
        assert_eq!(Err(()), greater_filter.filter(&short_message_data).await);
    }
}
