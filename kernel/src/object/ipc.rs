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
    tag: MessageInfo,
    msg: [usize; MSG_MAX_LENGTH],
    user_data: usize,
    caps_or_badges: [usize; MSG_MAX_EXTRA_CAPS],
    recv_cnode: usize,
    recv_index: usize,
    recv_depth: usize,
}
