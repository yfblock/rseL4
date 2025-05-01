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
