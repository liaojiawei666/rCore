use alloc::sync::Arc;

use crate::fs::make_pipe;
use crate::fs::open_file;
use crate::fs::OpenFlags;
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::processor::{current_task, current_user_token};
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive();
    if fd > inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}
pub fn sys_open(path: *const u8, flag: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flag).unwrap()) {
        let mut inner = task.inner_exclusive();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}
pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive();
    if fd > inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd] = None;
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}
pub fn sys_dup(fd:usize)->isize{
    let task = current_task().unwrap();
    let mut inner=task.inner_exclusive();
    if fd>=inner.fd_table.len(){
        return -1;
    }
    if inner.fd_table[fd].is_none(){
        return -1;
    }
    let new_fd=inner.alloc_fd();
    inner.fd_table[new_fd]=Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}
