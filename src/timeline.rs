use crate::event::Event;

use chronology::{EventTimestamp, gen_chronology};
pub use error::TimelineError;

mod chronology;
mod error;

#[derive(Debug)]
pub struct Timeline {
	/// Ambiguously ordered list of events
	events: Vec<Event>,
	/// Chronological list of every event start- and end point
	chronology: Vec<EventTimestamp>,
}

impl Timeline {
	pub fn try_new(events: Vec<Event>) -> Result<Self, TimelineError> {
		let chronology = gen_chronology(&events)?;
		Ok(Self { events, chronology })
	}
}
