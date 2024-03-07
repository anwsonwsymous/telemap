use crate::processing::data::DataHub;
use crate::processing::helpers::find_output_message_text;
use crate::processing::pipe::Pipe;
use regex::Regex;
use rust_tdlib::types::FormattedText;

/// Search and replace texts with regular expression
#[derive(Debug, Default, Clone)]
pub struct ReplaceRegexp {
    search: String,
    replace: String,
    all: bool,
    search_pattern: Option<Regex>,
}

impl ReplaceRegexp {
    pub fn builder() -> ReplaceRegexpBuilder {
        let inner = ReplaceRegexp::default();
        ReplaceRegexpBuilder { inner }
    }
}

impl Pipe for ReplaceRegexp {
    fn handle(&self, data: &mut DataHub) {
        if let Some(m) = &data.output {
            if let Some(formatted_text) = find_output_message_text(m) {
                let mut replaced_text = formatted_text.text().to_owned();

                if self.all {
                    replaced_text = self
                        .search_pattern
                        .as_ref()
                        .unwrap()
                        .replace_all(&replaced_text, &self.replace)
                        .to_string();
                } else {
                    replaced_text = self
                        .search_pattern
                        .as_ref()
                        .unwrap()
                        .replace(&replaced_text, &self.replace)
                        .to_string();
                }

                if formatted_text.text().eq(&replaced_text) {
                    return;
                }

                let new_formatted_text = FormattedText::builder()
                    .text(replaced_text)
                    .entities(formatted_text.entities().clone())
                    .build();

                data.set_output_text(new_formatted_text);
            }
        }
    }
}

pub struct ReplaceRegexpBuilder {
    inner: ReplaceRegexp,
}

impl ReplaceRegexpBuilder {
    pub fn search(&mut self, search: String) -> &mut ReplaceRegexpBuilder {
        self.inner.search = search;
        self.inner.search_pattern = Some(Regex::new(&self.inner.search).unwrap());
        self
    }

    pub fn replace(&mut self, replace: String) -> &mut ReplaceRegexpBuilder {
        self.inner.replace = replace;
        self
    }

    pub fn all(&mut self, all: bool) -> &mut ReplaceRegexpBuilder {
        self.inner.all = all;
        self
    }

    pub fn build(&self) -> ReplaceRegexp {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::PipeConf;
    use crate::processing::pipe::{Pipe, PipeType};
    use crate::processing::test_helpers::{formatted_text_example, transformed_data_example};
    use rust_tdlib::types::InputMessageContent;

    #[test]
    fn test_replace_regexp_all() {
        let mut data = transformed_data_example(Some("World world word work".to_string()));
        let success_text =
            formatted_text_example(Some("replaced replaced replaced replaced".to_string()));
        let pipe = PipeType::from(PipeConf::ReplaceRegexp {
            search: r"[wW]or\S+".to_string(),
            replace: "replaced".to_string(),
            all: true,
        });

        pipe.handle(&mut data);

        println!("{:?}", pipe);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(data_text.text(), success_text.text());
    }

    #[test]
    fn test_replace_regexp_first() {
        let mut data = transformed_data_example(Some("World world word work".to_string()));
        let success_text = formatted_text_example(Some("replaced world word work".to_string()));
        let pipe = PipeType::from(PipeConf::ReplaceRegexp {
            search: r"[wW]or\S+".to_string(),
            replace: "replaced".to_string(),
            all: false,
        });

        pipe.handle(&mut data);

        println!("{:?}", pipe);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(data_text.text(), success_text.text());
    }
}
