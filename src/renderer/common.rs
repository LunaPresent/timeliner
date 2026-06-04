use std::io;

use thiserror::Error;

use crate::timeline::Timeline;

#[derive(Debug, Error)]
pub enum RenderError {
	#[error("{0}")]
	WriteError(#[from] io::Error),
}

pub trait Renderer {
	fn render(&self, target: impl io::Write, timeline: &Timeline) -> Result<(), RenderError>;
}
