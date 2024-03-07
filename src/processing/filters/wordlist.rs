use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::find_input_message_text;

#[derive(Debug, Default, Clone)]
pub enum WordListType {
    #[default]
    BlackList,
    WhiteList,
}

/// This filter passes/rejects when message matches any word in wordlist
#[derive(Debug, Default, Clone)]
pub struct WordList {
    list_type: WordListType,
    words: Vec<String>,
}

impl WordList {
    pub fn builder() -> WordListBuilder {
        let inner = WordList::default();
        WordListBuilder { inner }
    }
}

impl Filter for WordList {
    fn filter(&self, data: &DataHub) -> FilterResult {
        let text = find_input_message_text(data.input.message())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let contains_any = self
            .words
            .iter()
            .any(|word| text.contains(&word.to_lowercase()));

        match self.list_type {
            WordListType::WhiteList => contains_any.then_some(()).ok_or(()),
            WordListType::BlackList => (!contains_any).then_some(()).ok_or(()),
        }
    }
}

pub struct WordListBuilder {
    inner: WordList,
}

impl WordListBuilder {
    pub fn words(&mut self, words: Vec<String>) -> &mut WordListBuilder {
        self.inner.words = words;
        self
    }

    pub fn list_type(&mut self, list_type: WordListType) -> &mut WordListBuilder {
        self.inner.list_type = list_type;
        self
    }

    pub fn build(&self) -> WordList {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    #[test]
    fn test_white_list() {
        let message_data1 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("hello".to_string())),
            false,
        ));
        let message_data2 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("world".to_string())),
            false,
        ));
        let message_data3 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("something else".to_string())),
            false,
        ));
        let filter = FilterType::from(FilterConf::WhiteList {
            words: vec!["hello".to_string(), "world".to_string()],
        });

        assert_eq!(Ok(()), filter.filter(&message_data1));
        assert_eq!(Ok(()), filter.filter(&message_data2));
        assert_eq!(Err(()), filter.filter(&message_data3));
    }

    #[test]
    fn test_black_list() {
        let message_data1 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("hello".to_string())),
            false,
        ));
        let message_data2 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("world".to_string())),
            false,
        ));
        let message_data3 = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("something else".to_string())),
            false,
        ));
        let filter = FilterType::from(FilterConf::BlackList {
            words: vec!["hello".to_string(), "world".to_string()],
        });

        assert_eq!(Err(()), filter.filter(&message_data1));
        assert_eq!(Err(()), filter.filter(&message_data2));
        assert_eq!(Ok(()), filter.filter(&message_data3));
    }
}
