use crate::config::FilterConf;
use crate::processing::data::DataHub;
use crate::processing::helpers::{
    cmp, find_input_message_duration, find_input_message_file, find_input_message_text,
};
#[cfg(feature = "storage")]
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use regex::Regex;
use rust_tdlib::types::MessageContent;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
#[cfg(feature = "storage")]
use std::sync::Mutex;

/// Filters return Ok/Err instead of true/false
pub type FilterResult = Result<(), ()>;

/// Trait to filter received messages before mapping
pub trait Filter {
    /// Check anything in received message and return Ok or Err
    fn filter(&self, data: &DataHub) -> FilterResult;
}

#[derive(Debug, Clone)]
pub enum FilterType {
    /// Filter only incoming messages
    Incoming(Incoming),
    /// Filter by counter. Counter will be decremented on message receive. When counter becomes 0
    /// filter will return Ok otherwise Err
    Counter(Counter),
    /// Filter by message type. Only video messages
    Video(Video),
    /// Filter by message type. Only photo messages
    Photo(Photo),
    /// Filter by message type. Only animation messages
    Animation(Animation),
    /// Filter by message type. Only document messages
    Document(Document),
    /// Filter by message type. Only file messages,
    /// This includes photo, video, animation and document messages.
    File(File),
    /// Filter by file size
    FileSize(FileSize),
    /// Filter by video/animation duration
    Duration(Duration),
    /// Filter by text/caption length
    TextLength(TextLength),
    /// Filter by text/caption where regular expression matches
    RegExp(RegExp),
    /// Filter duplicates, pass unique messages
    #[cfg(feature = "storage")]
    Unique(Unique),
}

impl Filter for FilterType {
    fn filter(&self, data: &DataHub) -> FilterResult {
        match self {
            Self::Incoming(f) => f.filter(data),
            Self::Counter(f) => f.filter(data),
            Self::Video(f) => f.filter(data),
            Self::Photo(f) => f.filter(data),
            Self::Document(f) => f.filter(data),
            Self::Animation(f) => f.filter(data),
            Self::File(f) => f.filter(data),
            Self::FileSize(f) => f.filter(data),
            Self::Duration(f) => f.filter(data),
            Self::TextLength(f) => f.filter(data),
            Self::RegExp(f) => f.filter(data),
            #[cfg(feature = "storage")]
            Self::Unique(f) => f.filter(data),
        }
    }
}

impl From<FilterConf> for FilterType {
    fn from(filter_conf: FilterConf) -> Self {
        match filter_conf {
            FilterConf::Incoming => FilterType::Incoming(Incoming),

            FilterConf::Counter { count } => {
                FilterType::Counter(Counter::builder().count(count).build())
            }

            FilterConf::FileSize { size, op } => {
                FilterType::FileSize(FileSize::builder().size(size).operator(op).build())
            }

            FilterConf::Duration { duration, op } => {
                FilterType::Duration(Duration::builder().duration(duration).operator(op).build())
            }

            FilterConf::TextLength { len, op } => {
                FilterType::TextLength(TextLength::builder().length(len).operator(op).build())
            }

            FilterConf::RegExp { exp } => {
                FilterType::RegExp(RegExp::builder().expression(exp).build())
            }

            FilterConf::Video => FilterType::Video(Video),

            FilterConf::Photo => FilterType::Photo(Photo),

            FilterConf::Document => FilterType::Document(Document),

            FilterConf::Animation => FilterType::Animation(Animation),

            FilterConf::File => FilterType::File(File),

            #[cfg(feature = "storage")]
            FilterConf::Unique => FilterType::Unique(Unique),
        }
    }
}

/// Filter only incoming messages
#[derive(Debug, Clone)]
pub struct Incoming;

impl Filter for Incoming {
    fn filter(&self, data: &DataHub) -> FilterResult {
        if !data.input.message().is_outgoing() {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Filter duplicates, pass unique messages
#[cfg(feature = "storage")]
#[derive(Debug, Default, Clone)]
pub struct Unique;

#[cfg(feature = "storage")]
impl Filter for Unique {
    fn filter(&self, data: &DataHub) -> FilterResult {
        lazy_static::lazy_static! {
            static ref STORE: Mutex<PickleDb> = Mutex::new(PickleDb::new("key-value.db", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json));
        }
        let mut db = STORE.lock().unwrap();

        // TODO: For other messages then text, check uniqueness by file_id

        if let Some(text) = find_input_message_text(data.input.message()) {
            let digest = format!("{:x}", md5::compute(text));

            if db.get::<bool>(&digest).is_some() {
                return Err(());
            }
            db.set(&digest, &true).unwrap();
        }

        Ok(())
    }
}

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
    fn filter(&self, data: &DataHub) -> FilterResult {
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
    fn filter(&self, data: &DataHub) -> FilterResult {
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

/// Filter by text/caption where regular expression matches
#[derive(Debug, Default, Clone)]
pub struct RegExp {
    exp: String,
    pattern: Option<Regex>,
}

impl RegExp {
    pub fn builder() -> RegExpBuilder {
        let inner = RegExp::default();
        RegExpBuilder { inner }
    }
}

impl Filter for RegExp {
    fn filter(&self, data: &DataHub) -> FilterResult {
        let text = find_input_message_text(data.input.message()).ok_or(())?;

        match self.pattern.as_ref().ok_or(())?.is_match(text) {
            true => Ok(()),
            false => Err(()),
        }
    }
}

pub struct RegExpBuilder {
    inner: RegExp,
}

impl RegExpBuilder {
    pub fn expression(&mut self, exp: String) -> &mut RegExpBuilder {
        self.inner.exp = exp;
        self.inner.pattern = Some(Regex::new(&self.inner.exp).unwrap());
        self
    }

    pub fn build(&self) -> RegExp {
        self.inner.clone()
    }
}

/// Filter only video messages
#[derive(Debug, Clone)]
pub struct Video;

impl Filter for Video {
    fn filter(&self, data: &DataHub) -> FilterResult {
        // Filter only video messages
        if let MessageContent::MessageVideo(_) = data.input.message().content() {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Filter only photo messages
#[derive(Debug, Clone)]
pub struct Photo;

impl Filter for Photo {
    fn filter(&self, data: &DataHub) -> FilterResult {
        // Filter only Photo messages
        if let MessageContent::MessagePhoto(_) = data.input.message().content() {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Filter only document messages
#[derive(Debug, Clone)]
pub struct Document;

impl Filter for Document {
    fn filter(&self, data: &DataHub) -> FilterResult {
        // Filter only Document messages
        if let MessageContent::MessageDocument(_) = data.input.message().content() {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Filter only Gif/Animation messages
#[derive(Debug, Clone)]
pub struct Animation;

impl Filter for Animation {
    fn filter(&self, data: &DataHub) -> FilterResult {
        if let MessageContent::MessageAnimation(_) = data.input.message().content() {
            Ok(())
        } else if let MessageContent::MessageDocument(doc) = data.input.message().content() {
            // Find animated document gif
            match doc.document().mime_type().as_ref() {
                "image/gif" => Ok(()),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

/// Filter messages which have any type of file in it
#[derive(Debug, Clone)]
pub struct File;

impl Filter for File {
    fn filter(&self, data: &DataHub) -> FilterResult {
        lazy_static::lazy_static! {
            static ref FILTERS: [FilterType;4] = [
                FilterType::Video(Video),
                FilterType::Photo(Photo),
                FilterType::Animation(Animation),
                FilterType::Document(Document),
            ];
        }

        FILTERS
            .iter()
            .any(|f| f.filter(data).is_ok())
            .then_some(())
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    #[test]
    fn test_incoming() {
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

        assert_eq!(Ok(()), filter.filter(&incoming_data));
        assert_eq!(Err(()), filter.filter(&outgoing_data));
    }

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

        let example_filter = FilterType::from(FilterConf::RegExp {
            exp: "^example$".to_string(),
        });
        let number_filter = FilterType::from(FilterConf::RegExp {
            exp: "^[0-9]+$".to_string(),
        });

        assert_eq!(Ok(()), example_filter.filter(&example_message_data));
        assert_eq!(Err(()), example_filter.filter(&number_message_data));

        assert_eq!(Ok(()), number_filter.filter(&number_message_data));
        assert_eq!(Err(()), number_filter.filter(&example_message_data));
    }

    #[test]
    fn test_text_length() {
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
        assert_eq!(Ok(()), less_filter.filter(&short_message_data));
        assert_eq!(Err(()), less_filter.filter(&long_message_data));

        // Equals to 5 symbols
        assert_eq!(Ok(()), eq_filter.filter(&short_message_data));
        assert_eq!(Err(()), eq_filter.filter(&long_message_data));

        // Greater than 5 symbols
        assert_eq!(Ok(()), greater_filter.filter(&long_message_data));
        assert_eq!(Err(()), greater_filter.filter(&short_message_data));
    }

    #[test]
    fn test_file_size() {
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

        assert_eq!(Ok(()), filter.filter(&video));
        assert_eq!(Err(()), filter.filter(&photo));
        assert_eq!(Ok(()), filter.filter(&document));
    }

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

    #[test]
    fn test_file() {
        let success_data = vec![
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Video(None, 0, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Document(None, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Photo(None, 0),
                false,
            )),
            DataHub::new(message_example(
                sender_user_example(),
                MessageMock::Animation(None, 0, 0),
                false,
            )),
        ];
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));

        let filter = FilterType::from(FilterConf::File);

        for datum in &success_data {
            assert_eq!(Ok(()), filter.filter(datum));
        }

        assert_eq!(Err(()), filter.filter(&fail_data));
    }

    #[test]
    fn test_animation() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Animation(None, 0, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Animation);

        assert_eq!(Ok(()), filter.filter(&success_data));
        assert_eq!(Err(()), filter.filter(&fail_data));
    }

    #[test]
    fn test_video() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Video(None, 0, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Video);

        assert_eq!(Ok(()), filter.filter(&success_data));
        assert_eq!(Err(()), filter.filter(&fail_data));
    }

    #[test]
    fn test_document() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Document(None, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Document);

        assert_eq!(Ok(()), filter.filter(&success_data));
        assert_eq!(Err(()), filter.filter(&fail_data));
    }

    #[test]
    fn test_photo() {
        let success_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Photo(None, 0),
            false,
        ));
        let fail_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(None),
            false,
        ));
        let filter = FilterType::from(FilterConf::Photo);

        assert_eq!(Ok(()), filter.filter(&success_data));
        assert_eq!(Err(()), filter.filter(&fail_data));
    }

    #[cfg(feature = "storage")]
    #[test]
    fn test_unique() {
        let original_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("some message".to_string())),
            false,
        ));
        let duplicate_message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("some message".to_string())),
            false,
        ));

        let unique_filter = FilterType::from(FilterConf::Unique);

        // Shorter than 10 symbols
        assert_eq!(Ok(()), unique_filter.filter(&original_message_data));
        assert_eq!(Err(()), unique_filter.filter(&duplicate_message_data));
    }
}
