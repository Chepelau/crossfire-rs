#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

pub mod backoff;
pub use backoff::Backoff;
#[cfg(feature = "padding")]
pub mod cache_padded;
#[cfg(feature = "padding")]
pub use cache_padded::CachePadded;
