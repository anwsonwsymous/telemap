use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::find_input_message_text;
use async_openai::types::{ChatCompletionResponseFormat, ChatCompletionResponseFormatType};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use strfmt::strfmt;
use tokio::sync::Mutex;

/// GPT parameters
const TEMPERATURE: f32 = 0.0f32;
const TOP_P: f32 = 0.0f32;
const MAX_TOKENS: u16 = 215u16;
const RESPONSE_FORMAT: ChatCompletionResponseFormat = ChatCompletionResponseFormat {
    r#type: ChatCompletionResponseFormatType::JsonObject,
};
const SYSTEM_PROMPT_TEMPLATE: &str = "
Given the following message and its context, evaluate its appropriateness, relevance, and adherence to predefined guidelines. Provide a decision on whether the message should be allowed or blocked.

Context Information:
'''
{context}
'''

Guidelines for Moderation:
'''
{guidelines}
'''

Based on the above information and guidelines, provide your analysis and decision.

IMPORTANT!!! Always return response in JSON format with 2 keys - analyses and allow.
analyses - MUST be text/string.
allow - MUST be true or false.
";

lazy_static! {
    static ref CLIENT: Mutex<Client<OpenAIConfig>> = Mutex::new(Client::new());
}

#[derive(Default, Deserialize)]
struct Response {
    #[serde(default)]
    pub allow: bool,
}

/// Filter by context using LLM
#[derive(Debug, Default, Clone)]
pub struct OpenAi {
    model: String,
    context_vars: HashMap<String, String>,
}

impl OpenAi {
    pub fn builder() -> OpenAiBuilder {
        let inner = OpenAi::default();
        OpenAiBuilder { inner }
    }
}

impl Filter for OpenAi {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        let input_text = find_input_message_text(data.input.message());
        if input_text.is_none() {
            return Err(());
        }

        let user_message = ChatCompletionRequestUserMessageArgs::default()
            .content(format!("User Message: '''\n{}\n'''", input_text.unwrap()))
            .build()
            .unwrap()
            .into();

        let system_message = ChatCompletionRequestSystemMessageArgs::default()
            .content(strfmt(SYSTEM_PROMPT_TEMPLATE, &self.context_vars).unwrap())
            .build()
            .unwrap()
            .into();

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .response_format(RESPONSE_FORMAT)
            .max_tokens(MAX_TOKENS)
            .temperature(TEMPERATURE)
            .top_p(TOP_P)
            .messages([system_message, user_message])
            .build()
            .unwrap();

        let client = CLIENT.lock().await;
        let response = client.chat().create(request).await;

        match response {
            Ok(response) => response
                .choices
                .iter()
                .any(|choice| {
                    serde_json::from_str::<Response>(
                        choice.message.content.as_ref().unwrap_or(&"{}".to_string()),
                    )
                    .unwrap()
                    .allow
                })
                .then_some(())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

pub struct OpenAiBuilder {
    inner: OpenAi,
}

impl OpenAiBuilder {
    pub fn context(&mut self, context: String) -> &mut OpenAiBuilder {
        self.inner
            .context_vars
            .insert("context".to_string(), context);
        self
    }

    pub fn guidelines(&mut self, guidelines: String) -> &mut OpenAiBuilder {
        self.inner
            .context_vars
            .insert("guidelines".to_string(), guidelines);
        self
    }

    pub fn model(&mut self, model: String) -> &mut OpenAiBuilder {
        self.inner.model = model;
        self
    }

    pub fn build(&self) -> OpenAi {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    fn openai_filter() -> FilterType {
        FilterType::from(FilterConf::OpenAi {
            model: "gpt-3.5-turbo".to_string(),
            context: r"
                Jobs/Vacancies. Only detailed vacancy/job messages, with title, position etc.
            "
            .to_string(),
            guidelines: r"
                1. No hate speech or discriminatory language.
                2. Messages must be relevant to the CONTEXT.
                3. No spam or promotional content.
            "
            .to_string(),
        })
    }

    #[ignore]
    #[tokio::test]
    async fn test_success_openai() {
        let data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some(
                r"
                Job Title: Senior Rust Developer

                Location: Fully Remote

                Company: TechStart, a cutting-edge technology company specializing in developing innovative software solutions.

                TechStart is seeking a talented and experienced Senior Rust Developer to join our growing team. As a Senior Rust Developer, you will be responsible for designing, implementing, and maintaining high-performance, reliable, and scalable software solutions using the Rust programming language.

                Key Responsibilities:
                - Collaborate with the engineering team to design and develop new features and enhancements in Rust
                - Write clean, efficient, and maintainable code following best practices
                - Optimize software performance and troubleshoot issues as they arise
                - Contribute to code reviews and provide constructive feedback to team members
                - Stay up-to-date with the latest Rust developments and technologies

                Requirements:
                - Bachelor's degree in Computer Science or related field
                - 5+ years of experience in software development using Rust
                - Strong understanding of data structures, algorithms, and software design principles
                - Experience with web development frameworks such as Actix or Rocket
                - Familiarity with containerization technologies like Docker and Kubernetes
                - Excellent problem-solving skills and attention to detail
                - Strong communication and teamwork abilities

                Preferred Qualifications:
                - Experience with distributed systems and microservices architecture
                - Knowledge of cloud computing platforms
                "
                    .to_string(),
            )),
            false,
        ));

        assert_eq!(Ok(()), openai_filter().filter(&data).await);
    }

    #[ignore]
    #[tokio::test]
    async fn test_fail_openai() {
        let data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some(
                r"
                Bitcoin is a digital currency created in 2009 by an unknown person or group of people using the name Satoshi Nakamoto. It operates on a decentralized network called blockchain, which allows transactions to be processed without the need for a central authority like a bank.

                Bitcoin has gained popularity as a form of investment and a means of payment for goods and services. Its value can be volatile, leading to both high potential returns and risks for investors.
                
                The process of obtaining bitcoin is known as mining, where individuals use powerful computers to solve complex mathematical puzzles. Bitcoin transactions are recorded on the blockchain, creating a transparent and secure ledger of all transactions.
                
                While bitcoin has faced criticism for its association with illegal activities and its environmental impact due to high energy consumption, it has also been praised for its potential to revolutionize the financial industry and increase financial inclusion for people around the world.
                "
                    .to_string(),
            )),
            false,
        ));

        assert_eq!(Err(()), openai_filter().filter(&data).await);
    }
}
