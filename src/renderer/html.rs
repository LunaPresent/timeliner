use std::io;

use super::{RenderError, Renderer};
use crate::date::GenericDate;
use crate::event::{Anchor, Event};
use crate::timeline::Timeline;

/// The stylesheet, bundled into the binary so `--output-format html-css` can emit
/// a fully standalone document.
const STYLE_CSS: &str = include_str!("../../template/style.css");

pub struct HtmlRenderer {
	pub bundle_css: bool,
}

/// Everything the layout pass needs to know about a single event, derived from
/// the chronology. Positions are ordinal (topological) row-line indices.
#[derive(Debug, Clone)]
struct Block<'a> {
	event: &'a Event,
	/// Resolved start date (may be empty if purely relative with no absolute anchor).
	start_date: &'a GenericDate,
	/// Resolved end date.
	end_date: &'a GenericDate,
	/// Whether the start anchor was specified absolutely (vs relative to another event).
	start_absolute: bool,
	/// Whether the end anchor was specified absolutely (vs relative to another event).
	end_absolute: bool,
	/// Inclusive start row-line position within the chronology.
	start_pos: usize,
	/// Exclusive end row-line position within the chronology.
	end_pos: usize,
	/// Swimlane column, assigned during layout. 0-based.
	column: usize,
	/// Whether start and end coincide (an "instant" event).
	instant: bool,
}

/// A point in the gutter where a date should be printed, keyed by row position.
struct DateMark<'a> {
	/// The chronology position of the timestamp this mark belongs to.
	position: usize,
	/// Whether this marks the end of a span. End positions are exclusive grid
	/// lines, so an end mark must render one row higher than its raw position to
	/// sit alongside the block's last occupied row rather than the row below it.
	is_end: bool,
	date: &'a GenericDate,
}

impl Renderer for HtmlRenderer {
	fn render(&self, mut target: impl io::Write, timeline: &Timeline) -> Result<(), RenderError> {
		let (blocks, columns, marks, rows) = layout(timeline);
		self.write_document(&mut target, &blocks, columns, &marks, rows)?;
		Ok(())
	}
}

/// Build the grid layout from a timeline.
///
/// Returns the positioned event blocks, the number of swimlane columns used, the
/// date marks for the gutter, and the total number of timestamp rows.
fn layout(timeline: &Timeline) -> (Vec<Block<'_>>, usize, Vec<DateMark<'_>>, usize) {
	let rows = timeline.iter().last().unwrap().position;

	// First pass: pair up each event's start and end timestamps into a single block.
	// We index blocks by the event's pointer identity within the timeline by walking
	// the chronology and matching anchors.
	//
	// Because the chronology is sorted, the first time we see an event it is its
	// start; the second time it is its end. (A start always sorts before its own end
	// thanks to the Anchor ordering, even for instantaneous events.)
	let mut blocks: Vec<Block> = Vec::with_capacity(rows / 2);
	// Map from event pointer -> index into `blocks` for the open (started) block.
	let mut open: Vec<(*const Event, usize)> = Vec::new();
	let mut marks: Vec<DateMark> = Vec::with_capacity(rows);

	for entry in timeline.iter() {
		let ptr = entry.event as *const Event;
		match entry.anchor {
			Anchor::Start => {
				// Only absolute occurrences carry a date that belongs to this event;
				// the date of a relative anchor is merely inherited from its target
				// and must not be printed in the gutter.
				if entry.is_absolute() {
					marks.push(DateMark {
						position: entry.position,
						is_end: false,
						date: entry.date,
					});
				}

				blocks.push(Block {
					event: entry.event,
					start_date: entry.date,
					end_date: entry.date,
					start_absolute: entry.is_absolute(),
					end_absolute: false, // patched on the matching End
					start_pos: entry.position,
					end_pos: entry.position, // patched on the matching End
					column: 0,
					instant: false,
				});
				open.push((ptr, blocks.len() - 1));
			}
			Anchor::End => {
				// Find the most recently opened block for this event.
				if let Some(slot) = open.iter().rposition(|(p, _)| *p == ptr) {
					let (_, idx) = open.remove(slot);
					blocks[idx].end_pos = entry.position;
					blocks[idx].end_date = entry.date;
					blocks[idx].end_absolute = entry.is_absolute();
					blocks[idx].instant = blocks[idx].end_pos == blocks[idx].start_pos + 1;

					// Emit a gutter date for the end anchor only when it is absolute
					// AND distinct from the start point. An instant event (same from
					// and until) would otherwise print its date twice.
					let same_point = blocks[idx].start_absolute
						&& blocks[idx].start_date == blocks[idx].end_date;
					if entry.is_absolute() && !same_point {
						marks.push(DateMark {
							position: entry.position,
							is_end: true,
							date: entry.date,
						});
					}
				}
			}
		}
	}

	// Second pass: greedy interval-graph colouring for the swimlanes.
	//
	// Blocks are already in start order (chronology is sorted and starts are pushed
	// in order). Two blocks may share a column only if they do not overlap; an event
	// ending exactly where another begins (touching) does NOT count as overlap, so
	// the column can be reused immediately.
	//
	// This naturally keeps earlier-started events in lower (left) columns among any
	// set of concurrent events, and packs non-concurrent events back to the left.
	let mut column_free_at: Vec<usize> = Vec::new(); // column -> first free position
	let mut order: Vec<usize> = (0..blocks.len()).collect();
	order.sort_by_key(|&i| (blocks[i].start_pos, blocks[i].end_pos));
	for &i in &order {
		let start = blocks[i].start_pos;
		let chosen = column_free_at
			.iter()
			.position(|&free_at| free_at <= start)
			.unwrap_or_else(|| {
				column_free_at.push(0);
				column_free_at.len() - 1
			});
		column_free_at[chosen] = blocks[i].end_pos;
		blocks[i].column = chosen;
	}

	let columns = column_free_at.len();
	(blocks, columns, marks, rows)
}

impl HtmlRenderer {
	fn write_document(
		&self,
		target: &mut impl io::Write,
		blocks: &[Block],
		columns: usize,
		marks: &[DateMark],
		rows: usize,
	) -> io::Result<()> {
		if self.bundle_css {
			writeln!(target, "<!DOCTYPE html>")?;
			writeln!(target, "<html lang=\"en\">")?;
			writeln!(target, "<head>")?;
			writeln!(target, "<meta charset=\"utf-8\">")?;
			writeln!(
				target,
				"<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
			)?;
			writeln!(target, "<title>Timeline</title>")?;
			writeln!(target, "<style>")?;
			target.write_all(STYLE_CSS.as_bytes())?;
			writeln!(target, "</style>")?;
			writeln!(target, "</head>")?;
			writeln!(target, "<body>")?;
		}

		self.write_timeline(target, blocks, columns, marks, rows)?;

		if self.bundle_css {
			writeln!(target, "</body>")?;
			writeln!(target, "</html>")?;
		}

		Ok(())
	}

	fn write_timeline(
		&self,
		target: &mut impl io::Write,
		blocks: &[Block],
		columns: usize,
		marks: &[DateMark],
		rows: usize,
	) -> io::Result<()> {
		writeln!(target, "<div class=\"tmln-root\">")?;

		if rows == 0 {
			writeln!(target, "<p class=\"tmln-empty\">No events to display.</p>")?;
			writeln!(target, "</div>")?;
			return Ok(());
		}

		// --tmln-cols drives how many swimlane columns the grid template defines.
		// --tmln-rows is informational; rows are placed by explicit line numbers.
		writeln!(
			target,
			"<div class=\"tmln-grid\" style=\"--tmln-cols: {columns}; --tmln-rows: {rows};\">"
		)?;

		// The vertical spine sits between the gutter and the swimlanes and runs the
		// full height of the grid.
		writeln!(
			target,
			"<div class=\"tmln-spine\" style=\"grid-row: 1 / {};\"></div>",
			rows + 1
		)?;

		// Date markers in the gutter. Each timestamp gets a tick; only print the
		// textual date when it is non-empty so relative-only points stay clean.
		for mark in marks {
			// A start mark sits at the block's first row (line position+1). An end
			// position is an exclusive grid line, so its mark renders one row higher
			// to align with the block's last occupied row.
			let line = if mark.is_end {
				mark.position
			} else {
				mark.position + 1
			};
			write!(
				target,
				"<div class=\"tmln-date\" style=\"grid-row: {line};\">"
			)?;
			write!(target, "<span class=\"tmln-tick\"></span>")?;
			let pretty = pretty_date(mark.date);
			if !pretty.is_empty() {
				write!(target, "<time class=\"tmln-date-label\">")?;
				write_escaped(target, &pretty)?;
				write!(target, "</time>")?;
			}
			writeln!(target, "</div>")?;
		}

		// Event blocks.
		for block in blocks {
			let row_start = block.start_pos + 1;
			let row_end = block.end_pos + 1;
			// Swimlane grid columns start at line 2 (line 1 is the gutter/spine).
			let col_line = block.column + 2;

			let instant_class = if block.instant {
				" tmln-event--instant"
			} else {
				" tmln-event--span"
			};

			writeln!(
				target,
				"<article class=\"tmln-event{instant_class}\" style=\"grid-row: {row_start} / {row_end}; grid-column: {col_line};\">"
			)?;

			write!(target, "<h3 class=\"tmln-event-title\">")?;
			write_escaped(target, &block.event.title)?;
			writeln!(target, "</h3>")?;

			// Per-event date range, shown inside the block so the topological layout
			// stays readable even though the gutter does not encode duration.
			let range = pretty_range(block);
			if !range.is_empty() {
				write!(target, "<p class=\"tmln-event-range\">")?;
				write_escaped(target, &range)?;
				writeln!(target, "</p>")?;
			}

			if let Some(body) = &block.event.body {
				write!(target, "<p class=\"tmln-event-body\">")?;
				write_escaped(target, body)?;
				writeln!(target, "</p>")?;
			}

			writeln!(target, "</article>")?;
		}

		writeln!(target, "</div>")?; // .tmln-grid
		writeln!(target, "</div>")?; // .tmln-root
		Ok(())
	}
}

/// Semi-pretty-print a [`GenericDate`].
///
/// The components are ordered largest-span first (year, month, day, hour, minute,
/// ...). We render the date part as `Y-MM-DD` (zero-padding the smaller fields)
/// and, if there are time components, append ` HH:MM[:SS]`.
fn pretty_date(date: &GenericDate) -> String {
	let c = &date.components;
	if c.is_empty() {
		return String::new();
	}

	let mut out = String::new();

	// Date portion: up to three components (year, month, day).
	let date_parts = c.len().min(3);
	for (i, comp) in c.iter().take(date_parts).enumerate() {
		if i == 0 {
			// Year keeps its sign and natural width.
			out.push_str(&comp.to_string());
		} else {
			out.push('-');
			// Month/day are zero-padded to two digits.
			out.push_str(&format!("{:02}", comp));
		}
	}

	// Time portion: anything beyond the third component.
	if c.len() > 3 {
		for (i, comp) in c.iter().skip(3).enumerate() {
			if i == 0 {
				out.push(' ');
			} else {
				out.push(':');
			}
			out.push_str(&format!("{:02}", comp));
		}
	}

	out
}

/// Pretty-print an event's start/end as a single human label.
///
/// Relative anchors carry only an inherited date, which is not this event's own,
/// so they are rendered as having no date at all.
fn pretty_range(block: &Block) -> String {
	let start_s = if block.start_absolute {
		pretty_date(block.start_date)
	} else {
		String::new()
	};
	let end_s = if block.end_absolute {
		pretty_date(block.end_date)
	} else {
		String::new()
	};

	// For a single-point block the two anchors denote the same instant, so collapse
	// to whichever date we actually have (the start, unless only the end is absolute).
	if block.instant || start_s == end_s {
		return if start_s.is_empty() { end_s } else { start_s };
	}

	match (start_s.is_empty(), end_s.is_empty()) {
		(false, false) => format!("{start_s} – {end_s}"),
		(false, true) => format!("from {start_s}"),
		(true, false) => format!("until {end_s}"),
		(true, true) => String::new(),
	}
}

/// Write `text` to `target`, escaping the characters that are unsafe in HTML
/// element content / attribute-free text.
fn write_escaped(target: &mut impl io::Write, text: &str) -> io::Result<()> {
	let mut last = 0;
	for (i, ch) in text.char_indices() {
		let replacement = match ch {
			'&' => "&amp;",
			'<' => "&lt;",
			'>' => "&gt;",
			'"' => "&quot;",
			'\'' => "&#39;",
			_ => continue,
		};
		target.write_all(&text.as_bytes()[last..i])?;
		target.write_all(replacement.as_bytes())?;
		last = i + ch.len_utf8();
	}
	target.write_all(&text.as_bytes()[last..])?;
	Ok(())
}
