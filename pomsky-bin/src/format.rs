use std::{
    fmt::Display,
    io::{StderrLock, Write},
};

use helptext::{Segment, Style};

use crate::Severity;

pub(crate) fn supports_color() -> bool {
    matches!(
        ::supports_color::on_cached(::supports_color::Stream::Stderr),
        Some(::supports_color::ColorLevel { has_basic: true, .. })
    )
}

pub(crate) struct Logger {
    colored: bool,
    enabled: bool,
}

impl Logger {
    fn copy(&self) -> Self {
        Logger { colored: self.colored, enabled: self.enabled }
    }

    pub(crate) fn new() -> Self {
        Logger { colored: supports_color(), enabled: true }
    }

    pub(crate) fn color(&self, colored: bool) -> Self {
        let mut copy = self.copy();
        copy.colored = colored;
        copy
    }

    pub(crate) fn enabled(self, enabled: bool) -> Self {
        let mut copy = self.copy();
        copy.enabled = enabled;
        copy
    }

    pub(crate) fn emptyln(&self) {
        if self.enabled {
            let mut buf = std::io::stderr().lock();
            let _ = buf.write_all(b"\n");
        }
    }

    pub(crate) fn basic(&self) -> Formatted<FormatBasic> {
        Formatted { format: FormatBasic, logger: self }
    }

    pub(crate) fn diagnostic<'a>(
        &self,
        severity: Severity,
        kind: &'a str,
    ) -> Formatted<FormatDetailed<'a, 4>> {
        let label = match severity {
            Severity::Error => Segment { style: Some(Style::R), text: "error", ticks: false },
            Severity::Warning => Segment { style: Some(Style::Y), text: "warning", ticks: false },
        };
        let start = [label, Segment::new("("), Segment::new(kind), Segment::new("):")];
        Formatted { format: FormatDetailed { start }, logger: self }
    }

    pub(crate) fn diagnostic_with_code<'a>(
        &self,
        severity: Severity,
        code: &'a str,
        kind: &'a str,
    ) -> Formatted<FormatDetailed<'a, 5>> {
        let label = match severity {
            Severity::Error => Segment { style: Some(Style::R), text: "error ", ticks: false },
            Severity::Warning => Segment { style: Some(Style::Y), text: "warning ", ticks: false },
        };
        let start = [
            label,
            Segment { style: Some(Style::R), text: code, ticks: false },
            Segment::new("("),
            Segment::new(kind),
            Segment::new("):"),
        ];
        Formatted { format: FormatDetailed { start }, logger: self }
    }

    pub(crate) fn error(&self) -> Formatted<FormatError> {
        Formatted { format: FormatError, logger: self }
    }

    pub(crate) fn warn(&self) -> Formatted<FormatWarning> {
        Formatted { format: FormatWarning, logger: self }
    }

    pub(crate) fn note(&self) -> Formatted<FormatNote> {
        Formatted { format: FormatNote, logger: self }
    }
}

pub(crate) struct FormatDetailed<'a, const N: usize> {
    start: [Segment<'a>; N],
}
pub(crate) struct FormatError;
pub(crate) struct FormatWarning;
pub(crate) struct FormatNote;
pub(crate) struct FormatBasic;

pub(crate) trait Format {
    fn start_segments(&self) -> &'_ [Segment<'_>];
    fn force_enabled() -> bool {
        false
    }
}

impl<const N: usize> Format for FormatDetailed<'_, N> {
    fn start_segments(&self) -> &[Segment<'_>] {
        &self.start
    }
}

const ERROR_SEGMENT: Segment<'_> =
    Segment { style: Some(Style::RedBold), ticks: false, text: "error" };
const WARN_SEGMENT: Segment<'_> =
    Segment { style: Some(Style::YellowBold), ticks: false, text: "warning" };
const NOTE_SEGMENT: Segment<'_> =
    Segment { style: Some(Style::CyanBold), ticks: false, text: "note" };
const END_SEGMENT: Segment<'_> = Segment { style: None, ticks: false, text: ": " };

impl Format for FormatError {
    fn start_segments(&self) -> &[Segment<'_>] {
        &[ERROR_SEGMENT, END_SEGMENT]
    }

    fn force_enabled() -> bool {
        true
    }
}

impl Format for FormatWarning {
    fn start_segments(&self) -> &[Segment<'_>] {
        &[WARN_SEGMENT, END_SEGMENT]
    }
}

impl Format for FormatNote {
    fn start_segments(&self) -> &[Segment<'_>] {
        &[NOTE_SEGMENT, END_SEGMENT]
    }
}

impl Format for FormatBasic {
    fn start_segments(&self) -> &[Segment<'_>] {
        &[]
    }
}

pub(crate) struct Formatted<'a, F: Format> {
    logger: &'a Logger,
    format: F,
}

impl<F: Format> Formatted<'_, F> {
    fn start(&self, buf: &mut StderrLock<'_>) {
        for segment in self.format.start_segments() {
            let _ = segment.write(buf, self.logger.colored, 0);
        }
    }

    pub(crate) fn print(&self, display: impl Display) {
        if self.logger.enabled || F::force_enabled() {
            let mut buf = std::io::stderr().lock();
            self.start(&mut buf);

            let _ = buf.write_fmt(format_args!("{display}"));
            buf.flush().unwrap();
        }
    }

    pub(crate) fn println(&self, display: impl Display) {
        if self.logger.enabled || F::force_enabled() {
            let mut buf = std::io::stderr().lock();
            self.start(&mut buf);

            let _ = buf.write_fmt(format_args!("{display}\n"));
        }
    }

    pub(crate) fn fmt(&self, segments: &[Segment]) {
        if self.logger.enabled || F::force_enabled() {
            let mut buf = std::io::stderr().lock();
            self.start(&mut buf);

            for segment in segments {
                let _ = segment.write(&mut buf, self.logger.colored, 0);
            }
            buf.flush().unwrap();
        }
    }

    pub(crate) fn fmtln(&self, segments: &[Segment]) {
        if self.logger.enabled || F::force_enabled() {
            let mut buf = std::io::stderr().lock();
            self.start(&mut buf);

            for segment in segments {
                let _ = segment.write(&mut buf, self.logger.colored, 0);
            }
            let _ = buf.write_all(b"\n");
        }
    }
}
