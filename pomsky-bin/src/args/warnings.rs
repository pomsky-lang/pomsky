use std::{ffi::OsString, str::FromStr};

use pomsky::diagnose::DiagnosticKind;

use super::ParseArgsError;

#[derive(Debug, PartialEq)]
pub(crate) enum DiagnosticSet {
    All,
    Disabled(Vec<DiagnosticKind>),
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

    pub(super) fn parse(value: OsString, mut warnings: Self) -> Result<Self, ParseArgsError> {
        let value = value.to_string_lossy();
        if value.as_ref() == "0" {
            Ok(DiagnosticSet::Enabled(vec![]))
        } else {
            let mut warning_list_own = vec![];
            let warning_list = match &mut warnings {
                DiagnosticSet::Disabled(set) => set,
                DiagnosticSet::Enabled(_) => return Ok(warnings),
                DiagnosticSet::All => &mut warning_list_own,
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
                    ParseArgsError::Other(format!(
                        "`{kind_str}` is not a recognized diagnostic kind"
                    ))
                })?;

                let (DiagnosticKind::Compat | DiagnosticKind::Deprecated) = kind else {
                return Err(ParseArgsError::WarningsNotAllowed(kind_str.to_string()))
            };

                warning_list.push(kind);
            }

            if matches!(warnings, DiagnosticSet::All) {
                Ok(DiagnosticSet::Disabled(warning_list_own))
            } else {
                Ok(warnings)
            }
        }
    }
}
