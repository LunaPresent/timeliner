use std::collections::HashMap;

use smallvec::SmallVec;

use super::TimelineError;
use crate::date::GenericDate;
use crate::event::{Anchor, Event, Occurence, RelativeOrder};

/// opaque struct purely used for correctly sorting before/after constraints
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RelativeSortKey {
	key: SmallVec<[RelativeOrder; 16]>,
}

impl Ord for RelativeSortKey {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let cmp_val = |v: Option<RelativeOrder>| match v {
			Some(RelativeOrder::Before) => -1,
			None => 0,
			Some(RelativeOrder::After) => 1,
		};

		for i in 0..self.key.len().max(other.key.len()) {
			let lhs = cmp_val(self.key.get(i).cloned());
			let rhs = cmp_val(other.key.get(i).cloned());
			if lhs == rhs {
				continue;
			} else {
				return lhs.cmp(&rhs);
			}
		}
		return std::cmp::Ordering::Equal;
	}
}

impl PartialOrd for RelativeSortKey {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(&other))
	}
}

/// two [EventTimestamp] objects are created for every event: a start and an end
/// this type only makes sense in a [super::Timeline] object alongside the matching event list
/// the order of the fields is important for sorting
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventTimestamp {
	/// The date of the event;
	/// OR that of the event which this references with before/after, recursively
	date: GenericDate,
	/// before/after sort key
	relative_sort_key: RelativeSortKey,
	/// defines whether this is the start or end of the event, makes "instant" events sort correctly
	anchor: Anchor,
	/// the index of the actual event in the matching events vec
	event_idx: usize,
}

pub fn gen_chronology(events: &[Event]) -> Result<Vec<EventTimestamp>, TimelineError> {
	let mut uid_to_idx: HashMap<&str, usize> = HashMap::new();
	for (i, e) in events.iter().enumerate() {
		if let Some(uid) = e.uid.as_ref()
			&& uid_to_idx.insert(uid, i).is_some()
		{
			return Err(TimelineError::DuplicateUid(uid.to_owned()));
		}
	}

	let mut chronology = events
		.iter()
		.enumerate()
		.flat_map(|(i, e)| {
			[
				EventTimestamp {
					date: e.from.date().cloned().unwrap_or_default(),
					relative_sort_key: Default::default(),
					anchor: Anchor::Start,
					event_idx: i,
				},
				EventTimestamp {
					date: e.until.date().cloned().unwrap_or_default(),
					relative_sort_key: Default::default(),
					anchor: Anchor::End,
					event_idx: i,
				},
			]
		})
		.collect::<Vec<_>>();

	let mut visit_map = vec![false; chronology.len()];
	for i in 0..chronology.len() {
		visit_map.fill(false);
		fill_timestamp(&mut chronology, i, events, &uid_to_idx, &mut visit_map)?;
	}

	chronology.sort_unstable();

	// TODO: remove printf debugging
	for et in &chronology {
		println!(
			"{} - {}",
			match et.anchor {
				Anchor::Start => "start",
				Anchor::End => "end  ",
			},
			events[et.event_idx].title
		);
	}
	// end of printf debugging

	Ok(chronology)
}

fn fill_timestamp(
	chronology: &mut [EventTimestamp],
	head: usize,
	events: &[Event],
	uid_to_idx: &HashMap<&str, usize>,
	visit_map: &mut [bool],
) -> Result<(), TimelineError> {
	if visit_map[head] {
		return Err(TimelineError::CyclicReference);
	}
	visit_map[head] = true;

	if !chronology[head].date.components.is_empty()
		|| !chronology[head].relative_sort_key.key.is_empty()
	{
		return Ok(());
	}

	let occurence = match chronology[head].anchor {
		Anchor::Start => &events[head / 2].from,
		Anchor::End => &events[head / 2].until,
	};

	if let Occurence::Relative {
		target_uid,
		order,
		anchor,
	} = occurence
	{
		let target_idx = *uid_to_idx
			.get(target_uid.as_str())
			.ok_or(TimelineError::UnknownUid(target_uid.to_owned()))?;
		let target_head = target_idx * 2
			+ match anchor {
				Anchor::Start => 0,
				Anchor::End => 1,
			};

		fill_timestamp(chronology, target_head, events, uid_to_idx, visit_map)?;

		chronology[head].relative_sort_key = chronology[target_head].relative_sort_key.clone();
		chronology[head].relative_sort_key.key.push(*order);
		chronology[head].date = chronology[target_head].date.clone();
	}

	Ok(())
}
