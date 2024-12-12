use pomsky_macro::pomsky;

#[test]
fn rust() {
    const REGEX: &str = pomsky! {
        // variables
        let number = '-'? [d]+;
        let op = ["+-*/"];
        number (op number)*
    };

    assert_eq!(REGEX, "-?\\d+(?:[*+\\-/]-?\\d+)*");
}

#[test]
fn pcre() {
    const REGEX: &str = pomsky!(
        #flavor = Pcre
        "foo" (!>> "bar")
    );

    assert_eq!(REGEX, "foo(?!bar)");
}

#[test]
fn composite_tokens() {
    const REGEX: &str = pomsky!(
        Start "Test" End
    );

    assert_eq!(REGEX, "^Test$");
}
