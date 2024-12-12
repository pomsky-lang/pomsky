use std::{ffi::OsString, str::FromStr};

use pomsky::diagnose::DiagnosticKind;

use super::ParseArgsError;

#[derive(Debug, PartialEq, Default)]
pub(crate) enum DiagnosticSet {
    #[default]
    All,
    Disabled(Vec<DiagnosticKind>),
    // the list is currently unused -- `-W0` disables all warnings, so the set of warnings is empty
    Enabled(Vec<DiagnosticKind>),
}

impl DiagnosticSet {
    pub(crate) fn is_enabled(&self, kind: DiagnosticKind) -> bool {
        match self {
            DiagnosticSet::All => true,
            DiagnosticSet::Disabled(set) => !set.contains(&kind),
            DiagnosticSet::Enabled(set) => set.contains(&kind),
        }
    }

    pub(super) fn parse(&mut self, value: OsString) -> Result<(), ParseArgsError> {
        let value = value.to_string_lossy();
        if value.as_ref() == "0" {
            *self = DiagnosticSet::Enabled(vec![]);
            return Ok(());
        }

        let warning_list = match self {
            DiagnosticSet::Disabled(set) => set,
            DiagnosticSet::Enabled(_) => return Ok(()),
            DiagnosticSet::All => {
                *self = DiagnosticSet::Disabled(vec![]);
                let DiagnosticSet::Disabled(warning_list) = self else { unreachable!() };
                warning_list
            }
        };

        for warning in value.split(',') {
            let (kind_str, val) = warning
                .trim_start()
                .rsplit_once('=')
                .ok_or_else(|| ParseArgsError::WarningsNoEquals(warning.to_string()))?;

            if val != "0" {
                return Err(ParseArgsError::WarningsNoZero(kind_str.to_string()));
            }

            let kind = DiagnosticKind::from_str(kind_str).map_err(|_| {
                ParseArgsError::Other(format!("`{kind_str}` is not a recognized diagnostic kind"))
            })?;

            let (DiagnosticKind::Compat | DiagnosticKind::Deprecated) = kind else {
                return Err(ParseArgsError::WarningsNotAllowed(kind_str.to_string()));
            };

            warning_list.push(kind);
        }

        Ok(())
    }
}
