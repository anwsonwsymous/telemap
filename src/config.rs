use rust_tdlib::types::FormattedText;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Main function which accepts path to the file and tries to read configs and deserialize it
pub fn read_configs(path: &Path) -> Option<Configs> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    match from_reader(reader) {
        Ok(configs) => Some(configs),
        Err(e) => {
            eprint!("Error : {}", e);
            None
        }
    }
}

/// Config file representation struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    /// Chat's mappings, with source and destinations
    pub maps: Vec<IdMapConf>,
    /// Chat's pipelines. How to transform and filter messages before sending to destinations
    #[serde(default)]
    pub pipelines: Vec<PipelineConf>,
}

/// Map struct of source and destinations chats.
/// This is used to create MappingsIndex.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdMapConf {
    /// Source chat
    #[serde(rename(serialize = "src", deserialize = "src"))]
    pub source: i64,
    /// Destination chats
    #[serde(rename(serialize = "dest", deserialize = "dest"), default)]
    pub destinations: Vec<i64>,
}

/// Routing configuration with optional source and destination, but one of them is required.
/// For incoming message there could be multiple routes, so there are rules
/// 1. route with src and dest - 1 priority
/// 2. route with only dest - 2 priority
/// 3. route with only src - 3 priority
///
/// The highest priority route will be used.
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteConf {
    #[serde(rename(serialize = "src", deserialize = "src"))]
    pub source: Option<i64>,
    #[serde(rename(serialize = "dest", deserialize = "dest"))]
    pub destination: Option<i64>,
}

/// Used for default routing. 0 -> 0 routing is for all chats which has not concrete routing specified.
impl Default for RouteConf {
    fn default() -> Self {
        RouteConf {
            source: Some(0),
            destination: Some(0),
        }
    }
}

/// One Pipeline representation struct. This is the routing from source to destination, with filters and pipes.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConf {
    /// Name which is used in logs
    pub name: String,
    /// Route with source and destination chat ids. This pipeline will be applied to only this route.
    #[serde(default)]
    pub route: RouteConf,
    /// List of filters that should run before pipelines
    #[serde(default)]
    pub filters: Vec<FilterConf>,
    /// List of pipelines
    #[serde(default)]
    pub pipes: Vec<PipeConf>,
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

/// All available Filters
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum FilterConf {
    Incoming,
    Text,
    Video,
    Photo,
    Animation,
    Document,
    AnyFile,
    Duration {
        duration: i32,
        op: String,
    },
    TextLength {
        len: u16,
        op: String,
    },
    Counter {
        #[serde(default)]
        count: u8,
    },
    FileSize {
        size: f32,
        op: String,
    },
    Regexp {
        exp: String,
    },
    BlackList {
        words: Vec<String>,
    },
    WhiteList {
        words: Vec<String>,
    },
    #[cfg(feature = "storage")]
    Unique,
    #[cfg(feature = "openai")]
    OpenAi {
        #[serde(default = "default_model")]
        model: String,
        #[serde(default)]
        context: String,
        #[serde(default)]
        guidelines: String,
    },
}

/// All available Pipes
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum PipeConf {
    Transform,
    Format {
        template: String,
    },
    StaticText {
        #[serde(default)]
        formatted_text: FormattedText,
    },
    StaticPhoto {
        path: String,
    },
    Replace {
        search: Vec<String>,
        #[serde(default)]
        replace: String,
    },
    ReplaceRegexp {
        search: String,
        #[serde(default)]
        replace: String,
        #[serde(default)]
        all: bool,
    },
}
