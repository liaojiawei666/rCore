#![no_std]
mod block_cache;
mod block_dev;
mod layout;
mod bitmap;
mod efs;
mod vfs;
extern crate alloc;
pub const BLOCK_SZ:usize=512;//512bytes => 4096 bits
use bitmap::Bitmap;
use block_cache::{get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
use layout::*;
pub use vfs::Inode;

