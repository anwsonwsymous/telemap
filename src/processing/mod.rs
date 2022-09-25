/// Contains received message and new message content builder
mod data;
/// Filters for received messages. Applied before pipes
mod filter;
mod helpers;
/// Data handling types
mod pipe;
/// Pipeline controls process from filtering to creating send message content builder
pub mod pipeline;

#[cfg(test)]
mod test_helpers;

pub(crate) use self::helpers::transform;
pub use self::pipeline::Pipeline;
