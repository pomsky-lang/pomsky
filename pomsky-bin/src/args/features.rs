use std::ffi::OsString;

use pomsky::features::PomskyFeatures;

use super::ParseArgsError;

pub(super) fn parse_features(value: OsString) -> Result<PomskyFeatures, ParseArgsError> {
    let lower = value.to_string_lossy().to_ascii_lowercase();

    let mut features = PomskyFeatures::new();
    for part in lower.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            match part {
                "grapheme" => features.grapheme(true),
                "numbered-groups" => features.numbered_groups(true),
                "named-groups" => features.named_groups(true),
                "atomic-groups" => features.atomic_groups(true),
                "references" => features.references(true),
                "lazy-mode" => features.lazy_mode(true),
                "ascii-mode" => features.ascii_mode(true),
                "ranges" => features.ranges(true),
                "variables" => features.variables(true),
                "lookahead" => features.lookahead(true),
                "lookbehind" => features.lookbehind(true),
                "boundaries" => features.boundaries(true),
                "regexes" => features.regexes(true),
                "dot" => features.dot(true),
                s => {
                    efprintln!(Y!"warning" ": unknown feature `" {s} "`");
                    features
                }
            };
        }
    }

    Ok(features)
}
