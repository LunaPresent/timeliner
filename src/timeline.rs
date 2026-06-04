use crate::date::GenericDate;
use crate::event::{Anchor, Event, Occurence};

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

	/// Iterate over every start- and end point in chronological order.
	///
	/// This is the only way to read a timeline; it deliberately exposes only the
	/// fields a renderer needs and keeps the internal sorting machinery private.
	pub fn iter(&self) -> TimelineIterator<'_> {
		TimelineIterator {
			timeline: self,
			head: 0,
			pos: 0,
		}
	}
}

/// A single point in the chronology, paired with the event it belongs to.
///
/// Only the fields a renderer actually needs are exposed; the internal sort keys
/// and event indices stay private to the [`Timeline`].
#[derive(Debug, Clone, Copy)]
pub struct TimelineEntry<'a> {
	/// Position of this timestamp within the chronology, `0..timeline.len()`.
	///
	/// This is a purely ordinal (topological) coordinate: the distance between two
	/// positions reflects how many timestamps lie between them, never how much real
	/// time elapsed.
	pub position: usize,
	/// Whether this entry marks the start or the end of [`Self::event`].
	pub anchor: Anchor,
	/// The resolved date of this timestamp.
	///
	/// For relative occurrences this is the date inherited from the referenced
	/// event, which may be empty (no components) if nothing absolute was reachable.
	pub date: &'a GenericDate,
	/// The event this timestamp belongs to.
	pub event: &'a Event,
}

impl TimelineEntry<'_> {
	/// Whether this anchor's occurrence was specified absolutely (an explicit
	/// date) rather than relative to another event.
	///
	/// [`Self::date`] is meaningful for sorting in both cases, but for relative
	/// occurrences it is merely inherited from the referenced event and should not
	/// be presented as this event's own date.
	pub fn is_absolute(&self) -> bool {
		let occurrence = match self.anchor {
			Anchor::Start => &self.event.from,
			Anchor::End => &self.event.until,
		};
		matches!(occurrence, Occurence::Absolute(_))
	}
}

pub struct TimelineIterator<'a> {
	timeline: &'a Timeline,
	head: usize,
	pos: usize,
}

impl<'a> Iterator for TimelineIterator<'a> {
	type Item = TimelineEntry<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let ts = self.timeline.chronology.get(self.head)?;
		let entry = TimelineEntry {
			position: self.pos,
			anchor: ts.anchor(),
			date: ts.date(),
			event: &self.timeline.events[ts.event_idx()],
		};
		self.head += 1;
		if let Some(ts_next) = self.timeline.chronology.get(self.head)
			&& !ts_next.is_equivalent(ts)
		{
			self.pos += 1;
		}

		Some(entry)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.timeline.chronology.len() - self.head;
		(remaining, Some(remaining))
	}
}

impl ExactSizeIterator for TimelineIterator<'_> {}
