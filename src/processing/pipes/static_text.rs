use crate::processing::data::DataHub;
use crate::processing::pipe::Pipe;
use rust_tdlib::types::FormattedText;

/// Sets static text on send message. On media content this will set "caption", otherwise "text"
#[derive(Debug, Default, Clone)]
pub struct StaticText {
    formatted_text: FormattedText,
}

impl StaticText {
    pub fn builder() -> StaticTextBuilder {
        let inner = StaticText::default();
        StaticTextBuilder { inner }
    }
}

impl Pipe for StaticText {
    fn handle(&self, data: &mut DataHub) {
        data.set_output_text(self.formatted_text.clone());
    }
}

pub struct StaticTextBuilder {
    inner: StaticText,
}

impl StaticTextBuilder {
    pub fn text(&mut self, text: FormattedText) -> &mut StaticTextBuilder {
        self.inner.formatted_text = text;
        self
    }

    pub fn build(&self) -> StaticText {
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
    fn test_static_text() {
        let mut data = transformed_data_example(None);
        let success_formatted_text = formatted_text_example(Some("Test Text".to_string()));
        let pipe = PipeType::from(PipeConf::StaticText {
            formatted_text: success_formatted_text.clone(),
        });
        pipe.handle(&mut data);

        println!("{:?}", success_formatted_text);
        println!("{:?}", pipe);

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(success_formatted_text.text(), data_text.text());
    }
}
