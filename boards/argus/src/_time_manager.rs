use messages::FormattedNaiveDateTime;

use crate::can_manager::CanManager;

pub struct TimeManager {
	pub time: Option<FormattedNaiveDateTime>,
}

impl TimeManager {
	pub fn new(time: Option<FormattedNaiveDateTime>) -> Self {
		TimeManager { time }
	}
}
