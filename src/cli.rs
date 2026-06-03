use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum DataFormat {
	Auto,
	Json,
	Toml,
}

#[derive(Parser, Debug, Clone, Copy)]
#[command(version, about)]
pub struct Cli {
	#[arg(value_enum, short, long, default_value_t = DataFormat::Auto)]
	pub format: DataFormat,
}
