use crate::config::{Configs, IdMapConf, PipelineConf};
use crate::processing::Pipeline;
use rust_tdlib::client::tdlib_client::TdJson;
use rust_tdlib::client::{Client, ClientState, SignalAuthStateHandler, Worker};
use rust_tdlib::tdjson;
use rust_tdlib::types::{
    AuthorizationState, GetChat, OptionValue, OptionValueBoolean, SendMessage, SetOption,
    TdlibParameters, Update,
};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::{env, io};
use tokio::sync::mpsc::{Receiver, Sender};

pub const APP_NAME: &str = "Telemap";
pub const APP_VERSION: &str = "1.0";
pub const APP_LANG: &str = "en";

pub type PipelineKey = (Option<i64>, Option<i64>);
pub type Map = HashMap<i64, Vec<i64>>;

/// App struct is entry point
#[derive(Debug)]
pub struct App {
    /// Index of mappings. Keys are "Source Chat" ids and values are lists of "Destination Chat" ids
    pub mappings_index: Arc<MappingsIndex>,
    /// Index of pipelines. Keys are "routes" (source_chat_id:dest_chat_id) and values are Pipelines
    pub pipelines_index: Arc<PipelinesIndex>,
}

impl From<Configs> for App {
    fn from(configs: Configs) -> Self {
        Self {
            mappings_index: Arc::new(MappingsIndex::from(configs.maps)),
            pipelines_index: Arc::new(PipelinesIndex::from(configs.pipelines)),
        }
    }
}

impl App {
    /// Entrypoint
    pub async fn start(&mut self) {
        // Set log level
        self.set_log_level();

        let (auth_sender, auth_receiver) = tokio::sync::mpsc::channel(10);
        let auth_handler = SignalAuthStateHandler::new(auth_receiver);

        // Create worker
        let mut worker = Worker::builder()
            .with_auth_state_handler(auth_handler)
            .with_channels_send_timeout(2_f64)
            .with_read_updates_timeout(2_f64)
            .build()
            .unwrap();

        // Start worker
        println!("Starting worker...");
        let mut waiter = worker.start();

        // Create client and main channel
        let (sender, receiver) = tokio::sync::mpsc::channel::<Box<Update>>(100);

        let client = Client::builder()
            .with_tdlib_parameters(self.build_parameters())
            .with_updates_sender(sender)
            .with_auth_state_channel(5)
            .build()
            .unwrap();

        // Two sends below are common for TDLib authorization flow.
        auth_sender.send("".to_string()).await.unwrap(); // empty encryption key
        auth_sender.send("".to_string()).await.unwrap(); // hack for forcing wait_auth_state_change work

        println!("Authentication...");
        let client = tokio::select! {
            c = worker.bind_client(client) => {
                match c {
                    Ok(cl) => cl,
                    Err(e) => panic!("{:?}", e)
                }
            }
            w = &mut waiter => panic!("{:?}", w)
        };

        self.process_authentication(&worker, &client, &auth_sender)
            .await;

        println!("Setting options, loading chats and waiting for updates...");
        tokio::join!(
            self.set_client_options(&client),
            self.load_chats(&client, &self.mappings_index),
            self.handle_updates(&client, receiver)
        );

        println!("Closing client...");
        client.stop().await.unwrap();

        // Wait for client state closed
        loop {
            if worker.wait_client_state(&client).await.unwrap() == ClientState::Closed {
                println!("Client closed...");
                break;
            }
        }
        println!("Stopping Worker...");
        worker.stop();
    }

    /// Handle incoming updates from Telegram
    async fn handle_updates(&self, client: &Client<TdJson>, mut receiver: Receiver<Box<Update>>) {
        let default_pipeline = vec![Pipeline::default()];

        loop {
            if let Update::NewMessage(new_message) = *receiver.recv().await.unwrap() {
                let source_chat_id = &new_message.message().chat_id();

                if let Some(destination_chats) = self.mappings_index.get(source_chat_id) {
                    for dest_chat_id in destination_chats {
                        let pipelines = self
                            .pipelines_index
                            .find(source_chat_id, dest_chat_id)
                            .unwrap_or(&default_pipeline);

                        log::info!("Pipelines found for received message: {:?}", pipelines);

                        let send_messages = pipelines
                            .iter()
                            .map(|p| p.handle(new_message.clone()))
                            .filter(|r| r.is_ok())
                            .map(|result| {
                                SendMessage::builder()
                                    .input_message_content(result.unwrap())
                                    .chat_id(*dest_chat_id)
                                    .build()
                            });

                        for send_message in send_messages {
                            match client.send_message(send_message).await {
                                Ok(_) => log::debug!("Message sent to {}", dest_chat_id),
                                Err(e) => log::error!("Message not sent: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Authentication process with signal auth handler
    async fn process_authentication(
        &mut self,
        worker: &Worker<SignalAuthStateHandler, TdJson>,
        client: &Client<TdJson>,
        auth_sender: &Sender<String>,
    ) {
        // Required params for authentication
        let mut phone = env::var("TELEGRAM_PHONE").unwrap();
        let mut password = env::var("TELEGRAM_PASSWORD").unwrap_or_else(|_| "".to_string());

        // TODO: function for output
        loop {
            println!("Auth state handler loop...");
            match worker.wait_auth_state_change(client).await {
                Ok(res) => {
                    match res {
                        Ok(state) => match state {
                            ClientState::Opened => {
                                println!("client authorized; can start interaction");
                                break;
                            }
                            _ => {
                                println!("Not authorized yet")
                            }
                        },
                        Err((err, auth_state)) => {
                            match &auth_state.authorization_state() {
                                AuthorizationState::WaitPhoneNumber(_) => {
                                    if phone.is_empty() {
                                        println!("Type phone number...");

                                        phone = match io::stdin().read_line(&mut phone) {
                                            Ok(_) => phone.trim().to_string(),
                                            Err(e) => panic!("Can not get input value: {:?}", e),
                                        };
                                    } else {
                                        println!("Phone number from ENV...");
                                    }

                                    // send correct phone number
                                    auth_sender.send(phone.clone()).await.unwrap();
                                    // and handle auth state manually again
                                    worker
                                        .handle_auth_state(auth_state.authorization_state(), client)
                                        .await
                                        .expect("can't handle it");
                                    // HACK
                                    auth_sender.send("".to_string()).await.unwrap();
                                }
                                AuthorizationState::WaitCode(_) => {
                                    println!("Type auth code...");

                                    let mut auth_code = String::new();
                                    auth_code = match io::stdin().read_line(&mut auth_code) {
                                        Ok(_) => auth_code.trim().to_string(),
                                        Err(e) => panic!("Can not get input value: {:?}", e),
                                    };

                                    // send correct auth_code from stdin
                                    auth_sender.send(auth_code).await.unwrap();
                                    // and handle auth state manually again
                                    worker
                                        .handle_auth_state(auth_state.authorization_state(), client)
                                        .await
                                        .expect("can't handle it");
                                    // HACK
                                    auth_sender.send("".to_string()).await.unwrap();
                                }
                                AuthorizationState::WaitPassword(_) => {
                                    if password.is_empty() {
                                        println!("Type password...");
                                        password = rpassword::read_password().unwrap();
                                    } else {
                                        println!("Password from ENV...");
                                    }

                                    // send correct password
                                    auth_sender.send(password.clone()).await.unwrap();
                                    // and handle auth state manually again
                                    worker
                                        .handle_auth_state(auth_state.authorization_state(), client)
                                        .await
                                        .expect("can't handle it");
                                    // HACK
                                    auth_sender.send("".to_string()).await.unwrap();
                                }
                                _ => {
                                    panic!(
                                        "state: {:?}, error: {:?}",
                                        auth_state.authorization_state(),
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    panic!("cannot wait for auth state changes: {}", err);
                }
            }
        }
    }

    /// Set telegram options
    async fn set_client_options(&self, client: &Client<TdJson>) {
        client
            .set_option(
                SetOption::builder()
                    .name("always_parse_markdown")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(true).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("always parse markdown option set...");
        client
            .set_option(
                SetOption::builder()
                    .name("disable_animated_emoji")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(true).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("disable animated emoji option set...");
        client
            .set_option(
                SetOption::builder()
                    .name("disable_persistent_network_statistics")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(true).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("disable persistent network statistics option set...");
        client
            .set_option(
                SetOption::builder()
                    .name("ignore_inline_thumbnails")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(true).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("ignore inline thumbnails option set...");
        client
            .set_option(
                SetOption::builder()
                    .name("is_location_visible")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(false).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("location option set...");
        client
            .set_option(
                SetOption::builder()
                    .name("online")
                    .value(OptionValue::Boolean(
                        OptionValueBoolean::builder().value(false).build(),
                    ))
                    .build(),
            )
            .await
            .unwrap();
        println!("online option set...");

        println!("Set Options Finished....");
    }

    /// Get chats from telegram.
    async fn load_chats(&self, client: &Client<TdJson>, mappings_index: &MappingsIndex) {
        let mut chats_set = HashSet::new();

        // Make unique destination chats
        for (_, dests) in mappings_index.iter() {
            for dest in dests {
                chats_set.insert(dest);
            }
        }

        // Get chats
        for dest in chats_set {
            let _ = client.get_chat(GetChat::builder().chat_id(*dest)).await;
            println!("Chat loaded {dest}");
        }
        println!("Load chats Finished...");
    }

    fn set_log_level(&self) {
        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .unwrap();

        tdjson::set_log_verbosity_level(log_level);
    }

    fn build_parameters(&self) -> TdlibParameters {
        // Variables from environment
        let api_id = env::var("TELEGRAM_API_ID")
            .expect("set TELEGRAM_API_ID in .env")
            .parse::<i32>()
            .unwrap();
        let api_hash = env::var("TELEGRAM_API_HASH").expect("set TELEGRAM_API_HASH in .env");
        let database =
            env::var("TELEGRAM_DATABASE").unwrap_or_else(|_| "telegram_database".to_string());

        TdlibParameters::builder()
            .api_id(api_id)
            .api_hash(&api_hash)
            .device_model(APP_NAME)
            .application_version(APP_VERSION)
            .system_language_code(APP_LANG)
            .database_directory(database)
            .use_secret_chats(true)
            .use_test_dc(false)
            .use_message_database(false)
            .use_file_database(false)
            .use_chat_info_database(false)
            .enable_storage_optimizer(false)
            .build()
    }
}

/// Routes/Mappings of chats. From source to multiple destinations.
#[derive(Debug, Clone)]
pub struct MappingsIndex {
    map: Map,
}

impl Deref for MappingsIndex {
    type Target = Map;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl From<Vec<IdMapConf>> for MappingsIndex {
    fn from(maps_conf: Vec<IdMapConf>) -> Self {
        let mut map = HashMap::new();

        for id_map in maps_conf {
            let destinations = if id_map.destinations.is_empty() {
                vec![id_map.source]
            } else {
                id_map.destinations
            };

            map.insert(id_map.source, destinations);
        }

        MappingsIndex { map }
    }
}

/// This struct contains indexed map of pipelines. Indexed by PipelineKey (source:dest)
#[derive(Debug, Clone)]
pub struct PipelinesIndex {
    map: HashMap<PipelineKey, Vec<Pipeline>>,
}

impl PipelinesIndex {
    pub fn find(&self, source: &i64, dest: &i64) -> Result<&Vec<Pipeline>, ()> {
        let mut index: PipelineKey = (Some(*source), Some(*dest));

        // Get for full route. (source, dest) key
        match self.map.get(&index) {
            Some(p) => Ok(p),
            None => {
                // Get for destination route. (None, dest) key
                index = (None, Some(*dest));
                match self.map.get(&index) {
                    Some(p) => Ok(p),
                    None => {
                        // Get for source route. (source, None) key
                        index = (Some(*source), None);
                        match self.map.get(&index) {
                            Some(p) => Ok(p),
                            None => {
                                // Get for source + dest with matching all *
                                if (source, dest) == (&0, &0) {
                                    Err(())
                                } else {
                                    self.find(&0, &0)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl From<Vec<PipelineConf>> for PipelinesIndex {
    fn from(pipelines_conf: Vec<PipelineConf>) -> Self {
        let mut map = HashMap::new();

        for pipeline_conf in pipelines_conf {
            let key = (pipeline_conf.route.source, pipeline_conf.route.destination);
            let vec: &mut Vec<Pipeline> = map.entry(key).or_default();
            vec.push(Pipeline::from(pipeline_conf));
        }

        PipelinesIndex { map }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{MappingsIndex, PipelinesIndex};
    use crate::config::{IdMapConf, PipelineConf, RouteConf};
    use std::collections::HashMap;

    fn mapping_example() -> MappingsIndex {
        MappingsIndex::from(vec![
            IdMapConf {
                source: 1,
                destinations: vec![10, 11],
            },
            IdMapConf {
                source: 2,
                destinations: vec![12, 13],
            },
            IdMapConf {
                source: 3,
                destinations: vec![],
            },
        ])
    }

    fn pipeline_conf_example(src: Option<i64>, dest: Option<i64>) -> PipelineConf {
        PipelineConf {
            name: "example pipeline".to_string(),
            route: RouteConf {
                source: src,
                destination: dest,
            },
            filters: vec![],
            pipes: vec![],
        }
    }

    #[test]
    fn test_mappings_index_get() {
        let mapping = mapping_example();

        assert_eq!(Some(&vec![10, 11]), mapping.get(&1));
        assert_eq!(Some(&vec![12, 13]), mapping.get(&2));
        assert_eq!(Some(&vec![3]), mapping.get(&3));
        assert_eq!(None, mapping.get(&4));
    }

    #[test]
    fn test_mappings_index_iter() {
        let mapping = mapping_example();

        let mut results = HashMap::new();
        results.insert(1, vec![10, 11]);
        results.insert(2, vec![12, 13]);
        results.insert(3, vec![3]);

        for (source, destinations) in mapping.iter() {
            assert_eq!(results.get(source).unwrap(), destinations);
        }
    }

    #[test]
    fn test_pipelines_index() {
        let pipelines = PipelinesIndex::from(vec![
            // With source and destination
            pipeline_conf_example(Some(1), Some(10)),
            // With destination only
            pipeline_conf_example(None, Some(10)),
            // With source only
            pipeline_conf_example(Some(1), None),
            // This will apply to all destinations for which no pipeline found
            pipeline_conf_example(Some(0), Some(0)),
        ]);

        assert!(pipelines.find(&1, &10).is_ok());
        assert!(pipelines.find(&20, &10).is_ok());
        assert!(pipelines.find(&1, &500).is_ok());
        assert!(pipelines.find(&1841, &895).is_ok());
    }

    #[test]
    fn test_multiple_pipelines_for_route() {
        let pipelines = PipelinesIndex::from(vec![
            pipeline_conf_example(Some(1), Some(10)),
            pipeline_conf_example(Some(1), Some(10)),
        ]);

        assert_eq!(2, pipelines.find(&1, &10).unwrap().len());
    }
}
