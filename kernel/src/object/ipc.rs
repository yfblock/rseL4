use crate::config::{MSG_MAX_EXTRA_CAPS, MSG_MAX_LENGTH};

use super::structures::MessageInfo;

#[repr(u8)]
pub enum EndPointState {
    Idle = 0,
    Send = 1,
    Recv = 2,
}

#[repr(u8)]
pub enum NotificationState {
    Idle = 0,
    Waiting = 1,
    Active = 2,
}

pub struct IPCBuffer {
    pub tag: MessageInfo,
    pub msg: [usize; MSG_MAX_LENGTH],
    pub user_data: usize,
    pub caps_or_badges: [usize; MSG_MAX_EXTRA_CAPS],
    pub recv_cnode: usize,
    pub recv_index: usize,
    pub recv_depth: usize,
}
