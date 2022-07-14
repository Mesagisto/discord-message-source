use crate::bot::DcFile;
use mesagisto_client::res::Res;
use color_eyre::eyre::Result;

pub trait ResExt {
  fn put_dc_image_id(&self, uid: &u64, file_id: &DcFile) -> Result<()>;
}
impl ResExt for Res {
  #[inline]
  fn put_dc_image_id(&self, uid: &u64, file_id: &DcFile) -> Result<()> {
    self.put_image_id(uid.to_be_bytes(), serde_cbor::to_vec(file_id)?);
    Ok(())
  }
}
