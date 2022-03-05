use rulex_macro::rulex;

#[test]
fn rust() {
    const REGEX: &str = rulex!(r" 'foo' | 'bar'? ");

    assert_eq!(REGEX, "foo|(?:bar)??");
}

#[test]
fn pcre() {
    const REGEX: &str = rulex!(r#" 'foo' (!>> 'bar') "#, flavor = Pcre);

    assert_eq!(REGEX, "foo(?!bar)");
}
