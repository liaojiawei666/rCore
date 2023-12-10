use crate::task::{SignalFlags, MAX_SIG};

/// Action for a signal
#[repr(C, align(16))]//16字节对齐，避免SignalAction分配在不同的页帧
#[derive(Debug, Clone, Copy)]
pub struct SignalAction {
    pub handler: usize,//信号处理函数地址
    pub mask: SignalFlags,//处理信号时屏蔽的信号
}

impl Default for SignalAction {
    fn default() -> Self {
        Self {
            handler: 0,
            mask: SignalFlags::from_bits(40).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct SignalActions {
    pub table: [SignalAction; MAX_SIG + 1],
}

impl Default for SignalActions {
    fn default() -> Self {
        Self {
            table: [SignalAction::default(); MAX_SIG + 1],
        }
    }
}
