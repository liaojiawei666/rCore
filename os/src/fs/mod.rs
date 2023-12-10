use crate::mm::UserBuffer;
mod inode;
mod stdio;
mod pipe;
pub use inode::{OSInode, ROOT_INODE,open_file,OpenFlags,list_apps};
pub use stdio::{Stdin, Stdout};
pub use pipe::make_pipe;
pub trait File:Send+Sync{
   /// If readable
   fn readable(&self) -> bool;
   /// If writable
   fn writable(&self) -> bool;
   /// Read file to `UserBuffer`
   fn read(&self, buf: UserBuffer) -> usize;
   /// Write `UserBuffer` to file
   fn write(&self, buf: UserBuffer) -> usize;
}