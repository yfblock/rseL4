use spin::Mutex;

use crate::object::tcb::DomainSchedule;

pub static KS_DOM_SCHEDULE_IDX: Mutex<usize> = Mutex::new(0);
pub static KS_DOM_SCHEDULE: Mutex<[DomainSchedule; 1]> = Mutex::new([DomainSchedule::new(0, 1)]);
