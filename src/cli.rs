use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum InputFormat {
	Auto,
	Json,
	Toml,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
	Html,
	HtmlCss,
}

#[derive(Parser, Debug, Clone, Copy)]
#[command(version, about)]
pub struct Cli {
	#[arg(value_enum, short, long, default_value_t = InputFormat::Auto)]
	pub input_format: InputFormat,
	#[arg(value_enum, short, long, default_value_t = OutputFormat::Html)]
	pub output_format: OutputFormat,
}
