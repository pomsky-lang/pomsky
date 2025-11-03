use crate::Outcome;

pub(crate) fn rust(regex: &str, test_strings: &[impl AsRef<str>]) -> Outcome {
    match regex::Regex::new(regex) {
        Ok(regex) => {
            for text in test_strings {
                let text = text.as_ref();
                if !regex.is_match(text) {
                    return Outcome::Error(format!("Test string didn't match: {text}"));
                }
            }
            Outcome::Success
        }
        Err(e) => Outcome::Error(e.to_string()),
    }
}

pub(crate) fn pcre(regex: &str, test_strings: &[impl AsRef<str>]) -> Outcome {
    match pcre2::bytes::RegexBuilder::new().utf(true).ucp(true).build(regex) {
        Ok(regex) => {
            for text in test_strings {
                let text = text.as_ref();
                let is_match = match regex.is_match(text.as_bytes()) {
                    Ok(is_match) => is_match,
                    Err(e) => return Outcome::Error(e.to_string()),
                };
                if !is_match {
                    return Outcome::Error(format!("Test string didn't match: {text}"));
                }
            }
            Outcome::Success
        }
        Err(e) => {
            let width = regex[0..e.offset().unwrap_or(0)].chars().count().saturating_sub(1);
            Outcome::Error(format!("{e}\n>\n> {}\n> {:width$}^", &regex, ""))
        }
    }
}

pub(crate) fn ruby(regex: &str, test_strings: &[impl AsRef<str>]) -> Outcome {
    match onig::Regex::new(regex) {
        Ok(regex) => {
            for text in test_strings {
                let text = text.as_ref();
                if !regex.is_match(text) {
                    return Outcome::Error(format!("Test string didn't match: {text}"));
                }
            }
            Outcome::Success
        }
        Err(e) => Outcome::Error(e.to_string()),
    }
}

#[cfg(feature = "re2")]
pub(crate) fn re2(regex: &str, test_strings: &[impl AsRef<str>]) -> Outcome {
    match re2::RE2::compile(regex, re2::Options::default()) {
        Ok(regex) => {
            for text in test_strings {
                let text = text.as_ref();
                if !regex.partial_match(text) {
                    return Outcome::Error(format!("Test string didn't match: {text}"));
                }
            }
            Outcome::Success
        }
        Err(e) => Outcome::Error(e.message),
    }
}

pub fn pcre_version() -> String {
    let (major, minor) = pcre2::version();
    format!("{major}.{minor}")
}

pub fn onig_version() -> String {
    onig::version()
}
