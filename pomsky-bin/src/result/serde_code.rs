use pomsky::diagnose::DiagnosticCode;
use serde::{
    Deserializer, Serializer,
    de::{Error, Expected, Unexpected, Visitor},
};

pub(super) fn serialize<S>(value: &Option<DiagnosticCode>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.collect_str(value),
        None => serializer.serialize_none(),
    }
}

pub(super) fn deserialize<'de, D>(d: D) -> Result<Option<DiagnosticCode>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(CodeVisitor).map(Some)
}

struct CodeVisitor;

impl<'de> Visitor<'de> for CodeVisitor {
    type Value = DiagnosticCode;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "an integer that is a valid diagnostic code")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.trim_start_matches('P')
            .parse::<u16>()
            .map_or_else(|_| Err(()), DiagnosticCode::try_from)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &ExpectedCode))
    }
}

struct ExpectedCode;

impl Expected for ExpectedCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "diagnostic code")
    }
}

#[test]
fn test_serde() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Example {
        #[serde(with = "self", skip_serializing_if = "Option::is_none", default)]
        pub code: Option<DiagnosticCode>,
    }

    let value = Example { code: Some(DiagnosticCode::CaptureInLet) };
    let serialized = serde_json::to_string(&value).unwrap();

    assert_eq!(&serialized, r#"{"code":"P0308"}"#);
    assert_eq!(serde_json::from_str::<Example>(&serialized).unwrap(), value);

    let value_empty = Example { code: None };
    let serialized_empty = serde_json::to_string(&value_empty).unwrap();

    assert_eq!(&serialized_empty, "{}");
    assert_eq!(serde_json::from_str::<Example>(&serialized_empty).unwrap(), value_empty);
}
