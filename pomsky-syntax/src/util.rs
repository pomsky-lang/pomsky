#[cfg(feature = "suggestions")]
pub fn find_suggestion<'a>(
    found: &str,
    options: impl Iterator<Item = &'a str>,
) -> Option<Box<str>> {
    options
        .map(|option| (option, strsim::jaro_winkler(option, found)))
        .max_by(|(_, score1), (_, score2)| f64::total_cmp(score1, score2))
        .filter(|&(_, score)| score >= 0.8)
        .map(|(option, _)| option.into())
}
