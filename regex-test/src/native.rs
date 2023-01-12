use crate::Outcome;

pub(crate) fn rust(regex: &str) -> Outcome {
    match regex::Regex::new(regex) {
        Ok(_) => Outcome::Success,
        Err(e) => Outcome::Error(e.to_string()),
    }
}

pub(crate) fn pcre(regex: &str) -> Outcome {
    match pcre2::bytes::RegexBuilder::new().utf(true).build(regex) {
        Ok(_) => Outcome::Success,
        Err(e) => {
            let width = regex[0..e.offset().unwrap_or(0)].chars().count();
            Outcome::Error(format!("{e}\n>\n> {}\n> {:width$}^", &regex, ""))
        }
    }
}

pub(crate) fn ruby(regex: &str) -> Outcome {
    match onig::Regex::new(regex) {
        Ok(_) => Outcome::Success,
        Err(e) => Outcome::Error(e.to_string()),
    }
}
