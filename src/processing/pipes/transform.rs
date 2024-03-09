use crate::processing::data::DataHub;
use crate::processing::pipe::Pipe;
use crate::processing::transform;

/// Transform received message into send message.
#[derive(Debug, Clone)]
pub struct Transform;

impl Pipe for Transform {
    async fn handle(&self, data: &mut DataHub) {
        // All type of messages (text, video, photo, animation, etc...)
        if let Ok(new_message) = transform(data.input.message()) {
            data.output = Some(new_message);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::processing::test_helpers::transformed_data_example;

    #[tokio::test]
    async fn test_transform() {
        let data = transformed_data_example(None).await;

        assert!(data.output.is_some());
    }
}
