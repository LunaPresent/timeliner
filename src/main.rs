use std::io;
use std::io::Read as _;

use clap::Parser;
use color_eyre::eyre;

use cli::{Cli, DataFormat};
use event::EventList;
use timeline::Timeline;

mod cli;
mod date;
mod event;
mod timeline;

fn main() -> eyre::Result<()> {
	color_eyre::install()?;
	let cli = Cli::parse();
	let input = read_input()?;
	let events = parse_input(&input, cli.format)?;
	let timeline = Timeline::try_new(events.events)?;
	Ok(())
}

fn read_input() -> io::Result<String> {
	let mut buf = String::new();
	io::stdin().read_to_string(&mut buf)?;
	Ok(buf)
}

fn parse_input(input: &str, format: DataFormat) -> eyre::Result<EventList> {
	let events = match format {
		DataFormat::Auto => toml::from_str(input).or_else(|_| serde_json::from_str(input))?,
		DataFormat::Json => serde_json::from_str(input)?,
		DataFormat::Toml => toml::from_str(input)?,
	};
	Ok(events)
}
