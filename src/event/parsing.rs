use serde::Deserialize;

use crate::date::GenericDate;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ParsedEvent {
	pub title: String,
	#[serde(default)]
	pub body: Option<String>,
	#[serde(default)]
	pub uid: Option<String>,

	#[serde(alias = "date")]
	pub from: ParsedOccurence,
	#[serde(default)]
	pub until: Option<ParsedOccurence>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParsedOccurence {
	Absolute(GenericDate),
	Relative(ParsedRelativeOccurence),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParsedRelativeOccurence {
	Before(ParsedRelativeOccurenceTarget),
	After(ParsedRelativeOccurenceTarget),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParsedRelativeOccurenceTarget {
	Uid(String),
	Anchor(ParsedRelativeOccurenceTargetAnchor),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParsedRelativeOccurenceTargetAnchor {
	StartOf(String),
	EndOf(String),
}
