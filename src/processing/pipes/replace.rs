use crate::processing::data::DataHub;
use crate::processing::helpers::find_output_message_text;
use crate::processing::pipe::Pipe;
use rust_tdlib::types::FormattedText;

/// Search and replace texts
#[derive(Debug, Default, Clone)]
pub struct Replace {
    search: Vec<String>,
    replace: String,
}

impl Replace {
    pub fn builder() -> ReplaceBuilder {
        let inner = Replace::default();
        ReplaceBuilder { inner }
    }
}

impl Pipe for Replace {
    fn handle(&self, data: &mut DataHub) {
        if let Some(m) = &data.output {
            if let Some(formatted_text) = find_output_message_text(m) {
                let mut replaced_text = formatted_text.text().to_owned();
                for search in &self.search {
                    replaced_text = replaced_text.replace(search, &self.replace);
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

pub struct ReplaceBuilder {
    inner: Replace,
}

impl ReplaceBuilder {
    pub fn search(&mut self, search: Vec<String>) -> &mut ReplaceBuilder {
        self.inner.search = search;
        self
    }

    pub fn replace(&mut self, replace: String) -> &mut ReplaceBuilder {
        self.inner.replace = replace;
        self
    }

    pub fn build(&self) -> Replace {
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
    fn test_replace() {
        let mut data = transformed_data_example(Some("Search Search".to_string()));
        let success_text = formatted_text_example(Some("Replaced Replaced".to_string()));
        let pipe = PipeType::from(PipeConf::Replace {
            search: vec!["Search".to_string()],
            replace: "Replaced".to_string(),
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
    fn test_replace_multiple() {
        let mut data = transformed_data_example(Some("Search1 Search2".to_string()));
        let success_text = formatted_text_example(Some("Replaced Replaced".to_string()));
        let pipe = PipeType::from(PipeConf::Replace {
            search: vec!["Search1".to_string(), "Search2".to_string()],
            replace: "Replaced".to_string(),
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
