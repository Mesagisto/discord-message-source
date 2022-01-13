use crate::CONFIG;
use crate::bot::BOT_CLIENT;
use crate::{bot::DcFile, ext::db::DbExt, ext::res::ResExt};
use arcstr::ArcStr;
use mesagisto_client::OptionExt;
use mesagisto_client::{
  data::{
    message::{self, MessageType, Profile},
    Packet,
  },
  db::DB,
  res::RES,
  server::SERVER,
  EitherExt,
};
use serenity::model::channel::Message;

use super::receive::receive_from_server;

pub async fn answer_common(msg: &Message) -> anyhow::Result<()> {
  let target = msg.channel_id.as_u64();
  if !CONFIG.target_address_mapper.contains_key(&target) {
    return Ok(());
  }
  let address = CONFIG.target_address_mapper.get(&target).unwrap().clone();
  let sender = &msg.author;
  let profile = Profile {
    id: sender.id.as_u64().to_be_bytes().to_vec(), // fixme
    username: Some(sender.name.clone()),
    nick: None,
  };
  let mut chain = Vec::<MessageType>::new();
  if !msg.content.is_empty() {
    chain.push(MessageType::Text {
      content: msg.content.clone(),
    })
  }
  // fixme image group
  if let Some(attach) = msg.attachments.get(0) {
    match &attach.content_type {
      Some(ty) => {
        if ty.starts_with("image") {
          let dc_file = DcFile::new(
            target,
            attach.id.as_u64(),
            &ArcStr::from(attach.filename.clone()),
          );
          RES.put_dc_image_id(attach.id.as_u64(), &dc_file)?;
          BOT_CLIENT.download_file(&dc_file).await?;
          chain.push(MessageType::Image {
            id: attach.id.as_u64().to_be_bytes().to_vec(),url:None
          });
          ()
        }
      }
      None => (),
    }
  }

  let reply = match &msg.referenced_message {
    Some(v) => {
      v.id.as_u64().to_be_bytes().to_vec().some()
    }
    None => None
  };
  // msg.attachments.get(0).unwrap().content_type;
  DB.put_msg_id_ir_0(&target, msg.id.as_u64(), msg.id.as_u64())?;
  let message = message::Message {
    profile,
    id: msg.id.as_u64().to_be_bytes().to_vec(),
    reply,
    chain,
  };
  let packet = Packet::from(message.tl())?;
  SERVER
    .send_and_receive(target.to_be_bytes().to_vec(), address, packet, receive_from_server)
    .await?;
  Ok(())
}
