use crate::processing::data::DataHub;
use crate::processing::helpers::find_output_message_text;
use crate::processing::pipe::Pipe;
use rust_tdlib::types::FormattedText;
use std::collections::HashMap;
use strfmt::strfmt;

/// Format send message by provided template
#[derive(Debug, Default, Clone)]
pub struct Format {
    template: String,
}

impl Format {
    pub fn builder() -> FormatBuilder {
        let inner = Format::default();
        FormatBuilder { inner }
    }
}

impl Pipe for Format {
    fn handle(&self, data: &mut DataHub) {
        if let Some(m) = &data.output {
            if let Some(formatted_text) = find_output_message_text(m) {
                // Available context variables: message, todo:source, todo:source_link
                let mut context_vars: HashMap<String, String> = HashMap::new();
                context_vars.insert("message".to_string(), formatted_text.text().to_string());

                data.set_output_text(
                    FormattedText::builder()
                        .text(strfmt(&self.template, &context_vars).unwrap())
                        .build(),
                );
            }
        }
    }
}

pub struct FormatBuilder {
    inner: Format,
}

impl FormatBuilder {
    pub fn template(&mut self, template: String) -> &mut FormatBuilder {
        self.inner.template = template;
        self
    }

    pub fn build(&self) -> Format {
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
    fn test_format() {
        let mut data = transformed_data_example(Some("Original message".to_string()));
        let success_text = formatted_text_example(Some("Start `Original message` End".to_string()));
        let pipe = PipeType::from(PipeConf::Format {
            template: "Start `{message}` End".to_string(),
        });

        pipe.handle(&mut data);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(data_text.text(), success_text.text());
    }
}
