use std::io::{self, Read as _, Write as _};

use clap::Parser;
use color_eyre::eyre;

use cli::{Cli, InputFormat, OutputFormat};
use event::EventList;
use renderer::Renderer as _;
use renderer::html::HtmlRenderer;
use timeline::Timeline;

mod cli;
mod date;
mod event;
mod renderer;
mod timeline;

fn main() -> eyre::Result<()> {
	color_eyre::install()?;
	let cli = Cli::parse();
	let input = read_input()?;
	let events = parse_input(&input, cli.input_format)?;
	let timeline = Timeline::try_new(events.events)?;

	let stdout = io::stdout();
	let mut writer = io::BufWriter::new(stdout.lock());
	// this obviously won't work if other renderers are added but that is a potentially never issue
	let renderer = match cli.output_format {
		OutputFormat::Html => HtmlRenderer { bundle_css: false },
		OutputFormat::HtmlCss => HtmlRenderer { bundle_css: true },
	};
	renderer.render(&mut writer, &timeline)?;
	writer.flush()?;

	Ok(())
}

fn read_input() -> io::Result<String> {
	let mut buf = String::new();
	io::stdin().read_to_string(&mut buf)?;
	Ok(buf)
}

fn parse_input(input: &str, format: InputFormat) -> eyre::Result<EventList> {
	let events = match format {
		InputFormat::Auto => toml::from_str(input).or_else(|_| serde_json::from_str(input))?,
		InputFormat::Json => serde_json::from_str(input)?,
		InputFormat::Toml => toml::from_str(input)?,
	};
	Ok(events)
}
