use mesagisto_client::data::{Packet, message::Message};

pub async fn receive_from_server(message: nats::asynk::Message, target: i64) -> anyhow::Result<()> {
  log::trace!("Receive from {}",target);
  let packet = Packet::from_cbor(&message.data)?;
  match packet {
    either::Left(msg) => {
      handle_receive_message(msg,target).await?;
    }
    either::Right(_) => {}
  }
  Ok(())
}

pub async fn handle_receive_message(mut message: Message, target: i64) -> anyhow::Result<()> {
  Ok(())
}