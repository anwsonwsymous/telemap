use crate::config::PipelineConf;
use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterType};
use crate::processing::filters::Incoming;
use crate::processing::pipe::{Pipe, PipeType};
use crate::processing::pipes::Transform;
use rust_tdlib::types::{InputMessageContent, UpdateNewMessage};
use std::error::Error;
use std::fmt;

/// Return type of pipeline
pub type PipelineResult = Result<InputMessageContent, PipelineError>;

#[derive(Debug)]
pub enum PipelineError {
    FilterError(String),
    OutputError(String),
}
impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipelineError::FilterError(message) => {
                write!(f, "Filter rejected: '{}'", message)
            }
            PipelineError::OutputError(message) => write!(f, "Output error: {}", message),
        }
    }
}
impl Error for PipelineError {}

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
        Pipeline {
            name: pipeline_conf.name,
            filters: pipeline_conf
                .filters
                .into_iter()
                .map(FilterType::from)
                .collect(),
            pipes: pipeline_conf
                .pipes
                .into_iter()
                .map(PipeType::from)
                .collect(),
        }
    }
}

impl Pipeline {
    pub fn handle(&self, input: UpdateNewMessage) -> PipelineResult {
        let mut data = DataHub::new(input);

        // First filter data
        self.filters.iter().try_for_each(|filter| {
            filter
                .filter(&data)
                .map_err(|_| PipelineError::FilterError(format!("{:?}", filter)))
        })?;

        // Then make output (run pipes)
        self.pipes.iter().for_each(|pipe| pipe.handle(&mut data));

        data.output.ok_or(PipelineError::OutputError(format!(
            "No output generated in pipeline {}",
            self.name
        )))
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
