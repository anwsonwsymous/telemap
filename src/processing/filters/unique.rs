use crate::processing::data::DataHub;
use crate::processing::filter::{Filter, FilterResult};
use crate::processing::helpers::find_input_message_text;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::path::Path;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref STORE: Mutex<PickleDb> = {
        let path = Path::new("storage/key-value.db");

        Mutex::new(if path.exists() {
            PickleDb::load_json(path, PickleDbDumpPolicy::AutoDump).expect("DB error")
        } else {
            PickleDb::new_json(path, PickleDbDumpPolicy::AutoDump)
        })
    };
}

/// Filter duplicates, pass unique messages
#[derive(Debug, Default, Clone)]
pub struct Unique;

impl Filter for Unique {
    async fn filter(&self, data: &DataHub) -> FilterResult {
        // TODO: For other messages then text, check uniqueness by file_id
        // TODO: Multiple unique filters for different pipelines with different PREFIX_KEY

        let mut db = STORE.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use crate::config::FilterConf;
    use crate::processing::data::DataHub;
    use crate::processing::filter::{Filter, FilterType};
    use crate::processing::test_helpers::{message_example, sender_user_example, MessageMock};

    #[tokio::test]
    async fn test_unique() {
        let message_data = DataHub::new(message_example(
            sender_user_example(),
            MessageMock::Text(Some("some message".to_string())),
            false,
        ));

        let unique_filter = FilterType::from(FilterConf::Unique);

        let _ = unique_filter.filter(&message_data).await;
        // Second time should not pass
        assert_eq!(Err(()), unique_filter.filter(&message_data).await);
    }
}
