use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Filter by counter
#[derive(Debug, Default, Clone)]
pub struct Counter {
    atomic: Arc<AtomicU8>,
    count: u8,
}

impl Counter {
    pub fn builder() -> CounterBuilder {
        let inner = Counter::default();
        CounterBuilder { inner }
    }
}

impl Filter for Counter {
    fn filter(&self, _: &DataHub) -> FilterResult {
        match self.atomic.fetch_sub(1, Ordering::Relaxed) {
            0 => {
                self.atomic.store(self.count, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(()),
        }
    }
}

pub struct CounterBuilder {
    inner: Counter,
}

impl CounterBuilder {
    pub fn count(&mut self, count: u8) -> &mut CounterBuilder {
        self.inner.count = count;
        self.inner.atomic = Arc::new(AtomicU8::from(count));
        self
    }

    pub fn build(&self) -> Counter {
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
    fn test_counter() {
        let data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));

        // This will pass only third message, first two must be ignored
        let filter = FilterType::from(FilterConf::Counter { count: 2 });

        assert_eq!(Err(()), filter.filter(&data));
        assert_eq!(Err(()), filter.filter(&data));
        assert_eq!(Ok(()), filter.filter(&data));
    }
}
