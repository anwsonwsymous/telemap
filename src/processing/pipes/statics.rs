use crate::processing::data::DataHub;
use crate::processing::helpers::transform_output_to_photo_message;
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
    async fn handle(&self, data: &mut DataHub) {
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

/// Sets static photo on send message
#[derive(Debug, Default, Clone)]
pub struct StaticPhoto {
    path: String,
}

impl StaticPhoto {
    pub fn builder() -> StaticPhotoBuilder {
        let inner = StaticPhoto::default();
        StaticPhotoBuilder { inner }
    }
}

impl Pipe for StaticPhoto {
    async fn handle(&self, data: &mut DataHub) {
        data.output = Some(transform_output_to_photo_message(
            data.output.as_ref().unwrap(),
            &self.path,
        ));
    }
}

pub struct StaticPhotoBuilder {
    inner: StaticPhoto,
}

impl StaticPhotoBuilder {
    pub fn path(&mut self, path: String) -> &mut StaticPhotoBuilder {
        self.inner.path = path;
        self
    }

    pub fn build(&self) -> StaticPhoto {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::PipeConf;
    use crate::processing::pipe::{Pipe, PipeType};
    use crate::processing::test_helpers::{formatted_text_example, transformed_data_example};
    use rust_tdlib::types::InputMessageContent;

    #[tokio::test]
    async fn test_static_text() {
        let mut data = transformed_data_example(None).await;
        let success_formatted_text = formatted_text_example(Some("Test Text".to_string()));
        let pipe = PipeType::from(PipeConf::StaticText {
            formatted_text: success_formatted_text.clone(),
        });
        pipe.handle(&mut data).await;

        let data_text = match data.output {
            Some(InputMessageContent::InputMessageText(m)) => m.text().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(success_formatted_text.text(), data_text.text());
    }

    #[tokio::test]
    async fn test_static_photo() {
        let mut data = transformed_data_example(Some("Something".to_string())).await;
        let success_formatted_text = formatted_text_example(Some("Something".to_string()));
        let pipe = PipeType::from(PipeConf::StaticPhoto {
            path: "resources/photo.jpg".to_string(),
        });
        pipe.handle(&mut data).await;

        let data_text = match data.output {
            Some(InputMessageContent::InputMessagePhoto(m)) => m.caption().clone(),
            _ => formatted_text_example(None),
        };

        assert_eq!(success_formatted_text.text(), data_text.text());
    }
}
