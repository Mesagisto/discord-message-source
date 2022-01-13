use crate::bot::DcFile;
use mesagisto_client::res::Res;

pub trait ResExt {
  fn put_dc_image_id(&self, uid: &u64, file_id: &DcFile) -> anyhow::Result<()>;
}
impl ResExt for Res {
  #[inline]
  fn put_dc_image_id(&self, uid: &u64, file_id: &DcFile) -> anyhow::Result<()> {
    self.put_image_id(uid.to_be_bytes(), serde_cbor::to_vec(file_id)?);
    Ok(())
  }
}
