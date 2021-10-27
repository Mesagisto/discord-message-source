use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static DATA: Lazy<RuntimeData> = Lazy::new(|| RuntimeData::default());

#[derive(Educe)]
#[educe(Default)]
pub struct RuntimeData {
  pub active_endpoint: DashMap<Arc<String>, bool>,
}
