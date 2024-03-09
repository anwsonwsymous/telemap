use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::{cmp, find_input_message_file};

/// Filter by File size in megabytes
#[derive(Debug, Default, Clone)]
pub struct FileSize {
    /// Size in MB
    size: f32,
    /// Operator (<,>,=)
    op: String,
}

impl FileSize {
    pub fn builder() -> FileSizeBuilder {
        let inner = FileSize::default();
        FileSizeBuilder { inner }
    }
}

impl Filter for FileSize {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        let file = find_input_message_file(data.input.message());

        match cmp(
            &self.op,
            &(file.ok_or(())?.expected_size() as f32 / 1000.0 / 1000.0),
            &self.size,
        ) {
            true => Ok(()),
            false => Err(()),
        }
    }
}

pub struct FileSizeBuilder {
    inner: FileSize,
}

impl FileSizeBuilder {
    pub fn size(&mut self, size: f32) -> &mut FileSizeBuilder {
        self.inner.size = size;
        self
    }

    pub fn operator(&mut self, op: String) -> &mut FileSizeBuilder {
        self.inner.op = op;
        self
    }

    pub fn build(&self) -> FileSize {
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
    async fn test_file_size() {
        // 10mb video
        let video = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Video(None, 10, 10 * 1000 * 1000),
            false,
        ));
        // 1mb photo
        let photo = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Photo(None, 1 * 1000 * 1000),
            false,
        ));
        // 20mb document
        let document = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Document(None, 20 * 1000 * 1000),
            false,
        ));

        let filter = FilterType::from(FilterConf::FileSize {
            size: 10 as f32,
            op: ">=".to_string(),
        });

        assert_eq!(Ok(()), filter.filter(&video).await);
        assert_eq!(Err(()), filter.filter(&photo).await);
        assert_eq!(Ok(()), filter.filter(&document).await);
    }
}
