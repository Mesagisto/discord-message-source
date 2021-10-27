use std::sync::Arc;
use crate::CONFIG;
use serenity::{client::Context, framework::standard::macros::hook, model::channel::Message};


pub async fn answer_common(msg: &Message)->anyhow::Result<()>{
  let target = msg.channel_id.as_u64();
  if !CONFIG.target_address_mapper.contains_key(&target) {
    return Ok(());
  }
  let address = CONFIG.target_address_mapper.get(&target).unwrap().clone();
  Ok(())

}