use crate::config::PipelineConf;
use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterType, Incoming};
use crate::processing::pipe::{Pipe, PipeType, Transform};
use rust_tdlib::types::{InputMessageContent, UpdateNewMessage};

/// Return type of pipeline
pub type PipelineResult = Result<InputMessageContent, ()>;

/// Contains the filters and output message building processes of mapping messages from one chat to another.
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// Pipeline name
    pub name: String,
    /// Filter messages
    pub filters: Vec<FilterType>,
    /// Make send message builder
    pub pipes: Vec<PipeType>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Pipeline {
            name: String::default(),
            filters: vec![FilterType::Incoming(Incoming)],
            pipes: vec![PipeType::Transform(Transform)],
        }
    }
}

impl From<PipelineConf> for Pipeline {
    fn from(pipeline_conf: PipelineConf) -> Self {
        let mut pipeline = Pipeline::default();

        for filter_conf in pipeline_conf.filters {
            pipeline.filters.push(FilterType::from(filter_conf))
        }

        for pipe_conf in pipeline_conf.pipes {
            pipeline.pipes.push(PipeType::from(pipe_conf));
        }

        pipeline.name = pipeline_conf.name;

        pipeline
    }
}

impl Pipeline {
    pub fn handle(&self, input: UpdateNewMessage) -> PipelineResult {
        let mut data = DataHub::new(input);

        // First filter data
        for filter in &self.filters {
            filter.filter(&data)?;
        }

        // Then make output (run pipes)
        for pipe in &self.pipes {
            pipe.handle(&mut data);
        }

        data.output.ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};
    use crate::processing::Pipeline;

    #[test]
    fn test_default() {
        // This must map any incoming message as it is
        let pipeline = Pipeline::default();
        let success_message =
            message_example(sender_user_example(), MessageMock::Text(None), false);
        let fail_message = message_example(sender_user_example(), MessageMock::Text(None), true);
        assert!(pipeline.handle(success_message).is_ok());
        assert!(pipeline.handle(fail_message).is_err());
    }
}
