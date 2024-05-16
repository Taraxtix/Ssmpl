use std::io::{IsTerminal, Write};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Level {
	Info,
	Warning,
	Error,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Report {
	level: Level,
	msg:   String,
}

impl Report {
	pub fn new(level: Level, msg: String) -> Self { Report { level, msg } }

	pub fn info(msg: String) -> Self {
		Report::new(Level::Info, format!("INFO: {}", msg))
	}

	pub fn warning(msg: String) -> Self {
		Report::new(Level::Warning, format!("WARN: {}", msg))
	}

	pub fn error(msg: String) -> Self {
		Report::new(Level::Error, format!("ERROR: {}", msg))
	}
}

pub struct Reporter {
	stdout:        StandardStream,
	pub min_level: Level,
	reports:       Vec<Report>,
}

impl Clone for Reporter {
	fn clone(&self) -> Self {
		let choice = if !std::io::stdout().is_terminal() {
			ColorChoice::Never
		} else {
			ColorChoice::Auto
		};
		let stdout = StandardStream::stdout(choice);
		Self { stdout, min_level: self.min_level.clone(), reports: self.reports.clone() }
	}
}

impl Reporter {
	pub fn new(min_level: Level) -> Self {
		let choice = if !std::io::stdout().is_terminal() {
			ColorChoice::Never
		} else {
			ColorChoice::Auto
		};
		let stdout = StandardStream::stdout(choice);
		Reporter { stdout, min_level, reports: Vec::new() }
	}

	pub fn report(&mut self, report: &Report) -> Result<&mut Self, std::io::Error> {
		let color = match report.level {
			| Level::Info => Color::Rgb(200, 200, 200),
			| Level::Warning => Color::Yellow,
			| Level::Error => Color::Red,
		};
		self.stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
		writeln!(self.stdout, "{}", report.msg)?;
		self.stdout.reset()?;
		Ok(self)
	}

	pub fn add(&mut self, report: Report) -> &mut Self {
		self.reports.push(report);
		self
	}

	pub fn add_error(&mut self, msg: String) -> &mut Self {
		self.add(Report::error(msg));
		self
	}

	pub fn add_warning(&mut self, msg: String) -> &mut Self {
		self.add(Report::warning(msg));
		self
	}

	pub fn add_info(&mut self, msg: String) -> &mut Self {
		self.add(Report::info(msg));
		self
	}

	pub fn flush(&mut self) -> &mut Self {
		let mut selected_reports = self
			.reports
			.iter()
			.filter_map(
				|x| if x.level >= self.min_level { Some(x.clone()) } else { None },
			)
			.collect::<Vec<_>>();
		selected_reports.sort_by_key(|x| x.level.clone());
		for report in selected_reports {
			self.report(&report).unwrap();
		}
		self.stdout.flush().unwrap();
		self
	}

	pub fn exit(&mut self, code: i32) -> ! {
		self.flush();
		std::process::exit(code)
	}

	pub fn exit_if(&mut self, level: Level, code: i32) -> &mut Self {
		if !self
			.reports
			.iter()
			.filter(|x| x.level >= level)
			.collect::<Vec<_>>()
			.is_empty()
		{
			self.exit(code)
		} else {
			self
		}
	}

	pub fn has(&self, level: Level) -> bool {
		self.reports.iter().any(|x| x.level >= level)
	}
}
