#[cfg(feature = "templating")]
pub mod format;
pub mod replace;
pub mod replace_regexp;
pub mod statics;
pub mod transform;

#[cfg(feature = "templating")]
pub(crate) use format::Format;
pub(crate) use replace::Replace;
pub(crate) use replace_regexp::ReplaceRegexp;
pub(crate) use statics::{StaticPhoto, StaticText};
pub(crate) use transform::Transform;
