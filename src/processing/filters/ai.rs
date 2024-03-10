use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::find_input_message_text;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use strfmt::strfmt;
use tokio::sync::Mutex;

const SYSTEM_PROMPT_TEMPLATE: &str = "
Given the following message and its context, evaluate its appropriateness, relevance, and adherence to predefined guidelines. Provide a decision on whether the message should be allowed or blocked.

Context Information:
'''
title: {title}
description: {description}
'''

Guidelines for Moderation:
'''
{guidelines}
'''

Based on the above information and guidelines, provide your analysis and decision in provided response format.

IMPORTANT!!! ONLY Answer with this Response Format: 
0 // Deny
1 // Allow
";

lazy_static! {
    static ref CLIENT: Mutex<Client<OpenAIConfig>> = Mutex::new(Client::new());
}

/// Filter by context using LLM
#[derive(Debug, Default, Clone)]
pub struct Context {
    model: String,
    context_vars: HashMap<String, String>,
}

impl Context {
    pub fn builder() -> ContextBuilder {
        let inner = Context::default();
        ContextBuilder { inner }
    }
}

impl Filter for Context {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1u16)
            .temperature(2.0f32)
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(strfmt(SYSTEM_PROMPT_TEMPLATE, &self.context_vars).unwrap())
                    .build()
                    .unwrap()
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(
                        find_input_message_text(data.input.message())
                            .unwrap()
                            .to_string(),
                    )
                    .build()
                    .unwrap()
                    .into(),
            ])
            .build()
            .unwrap();

        let client = CLIENT.lock().await;
        let response = client.chat().create(request).await.unwrap();

        response
            .choices
            .iter()
            .any(|choice| {
                choice
                    .message
                    .content
                    .as_ref()
                    .unwrap_or(&"0".to_string())
                    .parse::<u8>()
                    .unwrap_or(0)
                    == 1
            })
            .then_some(())
            .ok_or(())
    }
}

pub struct ContextBuilder {
    inner: Context,
}

impl ContextBuilder {
    pub fn title(&mut self, title: String) -> &mut ContextBuilder {
        self.inner.context_vars.insert("title".to_string(), title);
        self
    }

    pub fn description(&mut self, description: String) -> &mut ContextBuilder {
        self.inner
            .context_vars
            .insert("description".to_string(), description);
        self
    }

    pub fn guidelines(&mut self, guidelines: String) -> &mut ContextBuilder {
        self.inner
            .context_vars
            .insert("guidelines".to_string(), guidelines);
        self
    }

    pub fn model(&mut self, model: String) -> &mut ContextBuilder {
        self.inner.model = model;
        self
    }

    pub fn build(&self) -> Context {
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
    async fn test_context() {
        let success_data = DataHub::new(message_example(
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

        let fail_data = DataHub::new(message_example(
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

        // This will pass only third message, first two must be ignored
        let filter = FilterType::from(FilterConf::Context {
            model: "gpt-4-turbo-preview".to_string(),
            title: "Jobs/Vacancies".to_string(),
            description: r"
                Only detailed vacancy/job messages, with title, position etc.
            "
            .to_string(),
            guidelines: r"
                1. No hate speech or discriminatory language.
                2. Messages must be relevant to the CONTEXT.
                3. No spam or promotional content.
            "
            .to_string(),
        });

        assert_eq!(Ok(()), filter.filter(&success_data).await);
        assert_eq!(Err(()), filter.filter(&fail_data).await);
    }
}
