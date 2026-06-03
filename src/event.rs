use serde::Deserialize;
use serde_with::{FromInto, serde_as};

use crate::date::GenericDate;
use parsing::*;

mod parsing;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct EventList {
	#[serde_as(as = "Vec<FromInto<ParsedEvent>>")]
	pub events: Vec<Event>,
}

#[derive(Debug)]
pub struct Event {
	pub title: String,
	pub body: Option<String>,
	pub uid: Option<String>,
	pub from: Occurence,
	pub until: Occurence,
}

#[derive(Debug, Clone)]
pub enum Occurence {
	Absolute(GenericDate),
	Relative {
		target_uid: String,
		order: RelativeOrder,
		anchor: Anchor,
	},
}

impl Occurence {
	pub fn date(&self) -> Option<&GenericDate> {
		if let Occurence::Absolute(date) = self {
			Some(date)
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelativeOrder {
	Before,
	After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Anchor {
	Start,
	End,
}

impl From<ParsedEvent> for Event {
	fn from(value: ParsedEvent) -> Self {
		let from: Occurence = value.from.into();
		let until = if let Some(until) = value.until {
			until.into()
		} else {
			from.clone()
		};

		Self {
			title: value.title,
			body: value.body,
			uid: value.uid,
			from,
			until,
		}
	}
}

impl From<ParsedOccurence> for Occurence {
	fn from(value: ParsedOccurence) -> Self {
		match value {
			ParsedOccurence::Absolute(date) => Self::Absolute(date),
			ParsedOccurence::Relative(r) => {
				let (order, target) = match r {
					ParsedRelativeOccurence::Before(target) => (RelativeOrder::Before, target),
					ParsedRelativeOccurence::After(target) => (RelativeOrder::After, target),
				};
				let (anchor, target_uid) = match (target, order) {
					(ParsedRelativeOccurenceTarget::Uid(target_uid), RelativeOrder::Before) => {
						(Anchor::Start, target_uid)
					}
					(ParsedRelativeOccurenceTarget::Uid(target_uid), RelativeOrder::After) => {
						(Anchor::End, target_uid)
					}
					(
						ParsedRelativeOccurenceTarget::Anchor(
							ParsedRelativeOccurenceTargetAnchor::StartOf(target_uid),
						),
						_,
					) => (Anchor::Start, target_uid),
					(
						ParsedRelativeOccurenceTarget::Anchor(
							ParsedRelativeOccurenceTargetAnchor::EndOf(target_uid),
						),
						_,
					) => (Anchor::End, target_uid),
				};
				Self::Relative {
					target_uid,
					order,
					anchor,
				}
			}
		}
	}
}
