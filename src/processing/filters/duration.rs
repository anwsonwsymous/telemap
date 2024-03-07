use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::{cmp, find_input_message_duration};

/// Filter by media duration Video/Animation
#[derive(Debug, Default, Clone)]
pub struct Duration {
    duration: i32,
    /// Operator (<,>,=)
    op: String,
}

impl Duration {
    pub fn builder() -> DurationBuilder {
        let inner = Duration::default();
        DurationBuilder { inner }
    }
}

impl Filter for Duration {
    fn filter(&self, data: &DataHub) -> FilterResult {
        let duration = find_input_message_duration(data.input.message());

        match cmp(&self.op, &duration.ok_or(())?, &self.duration) {
            true => Ok(()),
            false => Err(()),
        }
    }
}

pub struct DurationBuilder {
    inner: Duration,
}

impl DurationBuilder {
    pub fn duration(&mut self, duration: i32) -> &mut DurationBuilder {
        self.inner.duration = duration;
        self
    }

    pub fn operator(&mut self, op: String) -> &mut DurationBuilder {
        self.inner.op = op;
        self
    }

    pub fn build(&self) -> Duration {
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
    fn test_duration() {
        // 10 seconds
        let animation = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Video(None, 10, 0),
            false,
        ));
        // 1 minute
        let video = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Video(None, 60, 0),
            false,
        ));

        let ten_seconds_filter = FilterType::from(FilterConf::Duration {
            duration: 10,
            op: "<=".to_string(),
        });
        let minute_filter = FilterType::from(FilterConf::Duration {
            duration: 60,
            op: ">=".to_string(),
        });

        assert_eq!(Ok(()), ten_seconds_filter.filter(&animation));
        assert_eq!(Err(()), ten_seconds_filter.filter(&video));

        assert_eq!(Ok(()), minute_filter.filter(&video));
        assert_eq!(Err(()), minute_filter.filter(&animation));
    }
}
