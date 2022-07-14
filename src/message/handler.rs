use serenity::{client::Context, framework::standard::macros::hook, model::channel::Message};
use tracing::error;

use crate::message::handlers::send::answer_common;

#[hook]
pub async fn message_hook(_: &Context, msg: &Message) {
  if let Err(e) = answer_common(msg).await {
    error!(
      "Error in message hook {} \n Backtrace {:?}",
      e,
      e.backtrace()
    )
  }
}
