use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimelineError {
	#[error("duplicate uid '{0}'")]
	DuplicateUid(String),
	#[error("unknown uid '{0}'")]
	UnknownUid(String),
	#[error("before/after constraints form a cycle — the timeline cannot be ordered")]
	CyclicReference,
}
