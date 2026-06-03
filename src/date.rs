use std::num::ParseIntError;
use std::str::FromStr;

use serde_with::DeserializeFromStr;
use smallvec::{SmallVec, smallvec};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenericDateError {
	#[error("nonempty string must contain at least one date component")]
	NoDateComponents,
	#[error("invalid date component {0}")]
	InvalidComponent(#[from] ParseIntError),
}

/// Represents an arbitrary calendar datetime with any amount of components
///
/// This is a strict superset of our own calendar, intended purely for sorting purposes
/// The components are ordered from largest timespan to shortest, just like yyyy-MM-dd
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, DeserializeFromStr)]
pub struct GenericDate {
	pub components: SmallVec<[i32; 4]>,
}

impl FromStr for GenericDate {
	type Err = GenericDateError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.trim();
		if s.is_empty() {
			return Ok(Self {
				components: smallvec![],
			});
		}

		let negative = if let Some(c) = s.chars().next() {
			c == '-'
		} else {
			return Err(GenericDateError::NoDateComponents);
		};
		let mut components = s
			.split(|c: char| !c.is_ascii_digit())
			.filter(|s| !s.is_empty())
			.map(|s| s.parse::<i32>())
			.collect::<Result<SmallVec<_>, ParseIntError>>()?;
		if let Some(comp) = components.first_mut() {
			if negative {
				*comp *= -1;
			}
		} else {
			return Err(GenericDateError::NoDateComponents);
		}

		Ok(Self { components })
	}
}
