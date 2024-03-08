use crate::config::FilterConf;
use crate::processing::data::DataHub;
use crate::processing::filters::{
    Counter, Duration, FileSize, Incoming, MessageType, Regexp, TextLength, Unique, WordList,
    WordListType,
};

/// Filters return Ok/Err instead of true/false
pub type FilterResult = Result<(), ()>;

/// Trait to filter received messages before mapping
pub trait Filter {
    fn filter(&self, data: &DataHub) -> FilterResult;
}

#[derive(Debug, Clone)]
pub enum FilterType {
    /// Filter only incoming messages
    Incoming(Incoming),
    /// Filter by counter. Counter will be decremented on message receive. When counter becomes 0
    /// filter will return Ok otherwise Err
    Counter(Counter),
    /// Only text messages
    Text(MessageType),
    /// Only video messages
    Video(MessageType),
    /// Only photo messages
    Photo(MessageType),
    /// Only animation messages
    Animation(MessageType),
    /// Only document messages
    Document(MessageType),
    /// Only file messages, includes photo, video, animation and document messages
    AnyFile(MessageType),
    /// Filter by file size
    FileSize(FileSize),
    /// Filter by video/animation duration
    Duration(Duration),
    /// Filter by text/caption length
    TextLength(TextLength),
    /// Filter by text/caption where regular expression matches
    Regexp(Regexp),
    /// This filter passes when message matches any word in wordlist
    WhiteList(WordList),
    /// This filter rejects when message matches any word in wordlist
    BlackList(WordList),
    /// Filter duplicates, pass unique messages
    #[cfg(feature = "storage")]
    Unique(Unique),
}

impl Filter for FilterType {
    fn filter(&self, data: &DataHub) -> FilterResult {
        match self {
            Self::Incoming(f) => f.filter(data),
            Self::Counter(f) => f.filter(data),
            Self::Text(f) => f.filter(data),
            Self::Video(f) => f.filter(data),
            Self::Photo(f) => f.filter(data),
            Self::Document(f) => f.filter(data),
            Self::Animation(f) => f.filter(data),
            Self::AnyFile(f) => f.filter(data),
            Self::FileSize(f) => f.filter(data),
            Self::Duration(f) => f.filter(data),
            Self::TextLength(f) => f.filter(data),
            Self::Regexp(f) => f.filter(data),
            Self::WhiteList(f) => f.filter(data),
            Self::BlackList(f) => f.filter(data),
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

            FilterConf::Regexp { exp } => {
                FilterType::Regexp(Regexp::builder().expression(exp).build())
            }

            FilterConf::WhiteList { words } => FilterType::WhiteList(
                WordList::builder()
                    .words(words)
                    .list_type(WordListType::WhiteList)
                    .build(),
            ),

            FilterConf::BlackList { words } => FilterType::BlackList(
                WordList::builder()
                    .words(words)
                    .list_type(WordListType::BlackList)
                    .build(),
            ),

            FilterConf::Text => FilterType::Text(MessageType::Text),

            FilterConf::Video => FilterType::Video(MessageType::Video),

            FilterConf::Photo => FilterType::Photo(MessageType::Photo),

            FilterConf::Document => FilterType::Document(MessageType::Document),

            FilterConf::Animation => FilterType::Animation(MessageType::Animation),

            FilterConf::AnyFile => FilterType::AnyFile(MessageType::AnyFile),

            #[cfg(feature = "storage")]
            FilterConf::Unique => FilterType::Unique(Unique),
        }
    }
}
