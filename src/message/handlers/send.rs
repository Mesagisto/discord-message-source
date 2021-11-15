use crate::ext::db::DbExt;
use crate::CONFIG;
use mesagisto_client::{EitherExt, data::{Packet, message::{self, MessageType, Profile}}, db::DB, res::RES, server::SERVER};
use serenity::{http::AttachmentType, model::channel::Message};

use super::receive::receive_from_server;


pub async fn answer_common(msg: &Message)->anyhow::Result<()>{
  let target = msg.channel_id.as_u64();
  if !CONFIG.target_address_mapper.contains_key(&target) {
    return Ok(());
  }
  let address = CONFIG.target_address_mapper.get(&target).unwrap().clone();
  let sender = &msg.author;
  let profile = Profile {
    id: sender.id.as_u64().to_be_bytes().to_vec(), // fixme
    username: Some(sender.name.clone()),
    nick: None
  };
  let mut chain = Vec::<MessageType>::new();
  if !msg.content.is_empty() {
    chain.push(MessageType::Text{ content: msg.content.clone() })
  }
  // fixme image group
  if let Some(attach) = msg.attachments.get(0) {
    match &attach.content_type {
        Some(ty) => if ty.starts_with("image") {
          // RES.store_photo_id()
          // attach.id.as_u64();
          // attach.url;
          ()
        },
        None => todo!(),
    }
  }
  dbg!(msg.attachments.clone());
  // msg.attachments.get(0).unwrap().content_type;
  DB.put_msg_id_ir_0(&target,msg.id.as_u64(),msg.id.as_u64())?;
  let message = message::Message {
    profile,
    id: msg.id.as_u64().to_be_bytes().to_vec(),
    reply: None,
    chain,
  };
  let packet = Packet::from(message.tl())?;
  SERVER.send_and_receive(*target as i64, address, packet, receive_from_server).await?;
  Ok(())
}