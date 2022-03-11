use rulex_macro::rulex;

#[test]
fn rust() {
    const REGEX: &str = rulex!("foo" | "bar"?);

    assert_eq!(REGEX, "foo|(?:bar)??");
}

#[test]
fn pcre() {
    const REGEX: &str = rulex!(
        #flavor = Pcre
        "foo" (!>> "bar")
    );

    assert_eq!(REGEX, "foo(?!bar)");
}
