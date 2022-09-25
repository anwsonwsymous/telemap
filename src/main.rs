mod app;
mod config;
mod processing;

use crate::app::App;
use crate::config::read_configs;
use argh::{from_env, FromArgs};
use dotenv::dotenv;
use std::path::Path;

#[derive(FromArgs)]
/// Allowed command line arguments
pub struct CliArgs {
    #[argh(option, short = 'c')]
    /// path to the configuration json file
    pub config_path: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let args: CliArgs = from_env();
    let mut app = App::from(read_configs(Path::new(&args.config_path)).unwrap());

    app.start().await;
}
