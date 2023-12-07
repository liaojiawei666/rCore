mod virtio_blk;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use easy_fs::BlockDevice;
pub use virtio_blk::VirtIOBlock;
type BlockDeviceImpl=VirtIOBlock;

lazy_static! {
    pub static ref BLOCK_DEVICE:Arc<dyn BlockDevice>=Arc::new(BlockDeviceImpl::new());
}