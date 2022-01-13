use std::convert::TryInto;

use mesagisto_client::{
  cache::CACHE,
  data::{
    message::{Message, MessageType},
    Packet,
  },
  db::DB,
};
use serenity::model::{
  channel::{AttachmentType, MessageReference},
  id::{ChannelId, MessageId},
};

use crate::bot::BOT_CLIENT;
use crate::config::CONFIG;
use crate::ext::db::DbExt;

pub async fn receive_from_server(
  message: nats::asynk::Message,
  target: Vec<u8>,
) -> anyhow::Result<()> {
  log::trace!("接收到目标{}的消息", base64_url::encode(&target));
  let packet = Packet::from_cbor(&message.data)?;
  match packet {
    either::Left(msg) => {
      handle_receive_message(msg, u64::from_be_bytes(target.try_into().unwrap())).await?;
    }
    either::Right(_) => {}
  }
  Ok(())
}

pub async fn handle_receive_message(mut message: Message, target_id: u64) -> anyhow::Result<()> {
  let target = BOT_CLIENT.get_channel(target_id).await?.id();
  for single in message.chain {
    let sender_name = if message.profile.nick.is_some() {
      message.profile.nick.take().unwrap()
    } else if message.profile.username.is_some() {
      message.profile.username.take().unwrap()
    } else {
      base64_url::encode(&message.profile.id)
    };
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
        DB.put_msg_id_ir_2(&target_id, &receipt.id.as_u64(), &message.id)?;
        let attachment = AttachmentType::File {
          file: &tokio::fs::File::open(&path).await.unwrap(),
          // 实际上图片不一定是png格式的. 这里使用png是为了欺骗Discord，让其识别其为图片.
          // 至于具体的MIME类型，对图片的显式不产生影响.
          // 经测试,支持的图片格式有JPG、PNG、WEBP
          // 动态WEBP会产生错误
          filename: format!("{:?}.png", path.file_name().unwrap()),
          // filename: path.file_name().unwrap().to_str().unwrap().to_string(),
        };
        let receipt = target
          .send_message(&**BOT_CLIENT, |m| {
            // m.content(format!("{}:", sender_name));
            m.add_file(attachment)
          })
          .await?;
        DB.put_msg_id_1(&target_id, &message.id, &receipt.id.as_u64())?;
      }
    }
  }
  Ok(())
}
