use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::find_input_message_text;
use regex::Regex;

/// Filter by text/caption where regular expression matches
#[derive(Debug, Default, Clone)]
pub struct Regexp {
    exp: String,
    pattern: Option<Regex>,
}

impl Regexp {
    pub fn builder() -> RegexpBuilder {
        let inner = Regexp::default();
        RegexpBuilder { inner }
    }
}

impl Filter for Regexp {
    fn filter(&self, data: &DataHub) -> FilterResult {
        let text = find_input_message_text(data.input.message()).ok_or(())?;

        match self.pattern.as_ref().ok_or(())?.is_match(text) {
            true => Ok(()),
            false => Err(()),
        }
    }
}

pub struct RegexpBuilder {
    inner: Regexp,
}

impl RegexpBuilder {
    pub fn expression(&mut self, exp: String) -> &mut RegexpBuilder {
        self.inner.exp = exp;
        self.inner.pattern = Some(Regex::new(&self.inner.exp).unwrap());
        self
    }

    pub fn build(&self) -> Regexp {
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
    fn test_regexp() {
        let example_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("example".to_string())),
            false,
        ));
        let number_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("123".to_string())),
            false,
        ));

        let example_filter = FilterType::from(FilterConf::Regexp {
            exp: "^example$".to_string(),
        });
        let number_filter = FilterType::from(FilterConf::Regexp {
            exp: "^[0-9]+$".to_string(),
        });

        assert_eq!(Ok(()), example_filter.filter(&example_message_data));
        assert_eq!(Err(()), example_filter.filter(&number_message_data));

        assert_eq!(Ok(()), number_filter.filter(&number_message_data));
        assert_eq!(Err(()), number_filter.filter(&example_message_data));
    }
}
