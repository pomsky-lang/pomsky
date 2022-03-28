use rulex_macro::rulex;

#[test]
fn rust() {
    const REGEX: &str = rulex! {
        // variables
        let number = '-'? [d]+;
        let op = ["+-*/"];
        number (op number)*
    };

    assert_eq!(REGEX, "-?\\d+(?:[+\\-*/]-?\\d+)*");
}

#[test]
fn pcre() {
    const REGEX: &str = rulex!(
        #flavor = Pcre
        "foo" (!>> "bar")
    );

    assert_eq!(REGEX, "foo(?!bar)");
}
