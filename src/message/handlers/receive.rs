use arcstr::ArcStr;
use mesagisto_client::{
  cache::CACHE,
  data::{
    message::{Message, MessageType},
    Packet,
  },
  db::DB, server::SERVER,
};

use serenity::model::{
  channel::{MessageReference, AttachmentType},
  id::{ChannelId, MessageId},
};
use tracing::trace;
use color_eyre::eyre::Result;

use crate::bot::BOT_CLIENT;
use crate::config::CONFIG;
use crate::ext::db::DbExt;

pub async fn recover() -> Result<()> {
  for pair in &CONFIG.bindings {
    SERVER.recv(
      ArcStr::from(pair.key().to_string()),
      pair.value(),
      server_msg_handler
    ).await?;
  }
  Ok(())
}
pub async fn add(target:u64,address: &ArcStr) -> Result<()> {
  SERVER.recv(
    target.to_string().into(),
    address,
    server_msg_handler
  ).await?;
  Ok(())
}
pub async fn change(target:u64,address: &ArcStr) -> Result<()> {
  SERVER.unsub(&target.to_string().into());
  add(target, address).await?;
  Ok(())
}
pub async fn del(target: u64) -> Result<()> {
  SERVER.unsub(&target.to_string().into());
  Ok(())
}

pub async fn server_msg_handler(
  message: nats::Message,
  target: ArcStr,
) -> Result<()> {
  trace!("接收到目标{}的消息", base64_url::encode(&target));
  let target = target.as_str().parse::<u64>()?;
  let packet = Packet::from_cbor(&message.payload)?;
  match packet {
    either::Left(msg) => {
      left_sub_handler(msg,target).await?;
    }
    either::Right(_) => {}
  }
  Ok(())
}

async fn left_sub_handler(mut message: Message, target_id: u64) -> Result<()> {
  let target = BOT_CLIENT.get_channel(target_id).await?.id();
  let sender_name = if message.profile.nick.is_some() {
    message.profile.nick.take().unwrap()
  } else if message.profile.username.is_some() {
    message.profile.username.take().unwrap()
  } else {
    base64_url::encode(&message.profile.id)
  };
  for single in message.chain {
    match single {
      MessageType::Text { content } => {
        let content = format!("{}: {}", sender_name, content);
        let receipt = if let Some(reply_to) = &message.reply {
          let local_id = DB.get_msg_id_1(&target_id, reply_to)?;
          match local_id {
            Some(local_id) => {
              let refer = MessageReference::from((ChannelId(target_id), MessageId::from(local_id)));
              target
                .send_message(&**BOT_CLIENT, |m| {
                  m.reference_message(refer).content(content)
                })
                .await
            }
            None => {
              target
                .send_message(&**BOT_CLIENT, |m| m.content(content))
                .await
            }
          }
        } else {
          target
            .send_message(&**BOT_CLIENT, |m| m.content(content))
            .await
        }?;
        DB.put_msg_id_1(&target_id, &message.id, receipt.id.as_u64())?;
      }
      MessageType::Image { id, url } => {
        let channel = CONFIG.mapper(&target_id).expect("Channel don't exist");
        let path = CACHE.file(&id, &url, &channel).await?;
        let receipt = target
          .send_message(&**BOT_CLIENT, |m| m.content(format!("{}:", sender_name)))
          .await?;
        DB.put_msg_id_ir_2(&target_id, receipt.id.as_u64(), &message.id)?;
        let kind = infer::get_from_path(&path).expect("file read failed when refering file type");

        let filename = match kind {
          Some(ty) => format!("{:?}.{}", path.file_name().unwrap(), ty.extension()),
          None => path.file_name().unwrap().to_string_lossy().to_string(),
        };
        let attachment = AttachmentType::File {
          file: &tokio::fs::File::open(&path).await.unwrap(),
          filename,
        };
        let receipt = target
          .send_message(&**BOT_CLIENT, |m| m.add_file(attachment))
          .await?;
        DB.put_msg_id_1(&target_id, &message.id, receipt.id.as_u64())?;
      }
      _ => {}
    }
  }
  Ok(())
}
