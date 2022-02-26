#[cfg(test)]
#[macro_use]
mod test_util;

mod parsers;
mod token;
mod tokens;

//mod tokens2;

pub(crate) use parsers::parse;
pub use token::{ParseErrorMsg, Token};
pub(crate) use tokens::Tokens;

#[cfg(test)]
mod tests {
    use crate::parse::parsers::*;

    #[test]
    fn string() {
        test!(parse_string "'test'" {
            lit!("test")
        });
        test!(parse_string r#""""# {
            lit!("")
        });
        test!(parse_string r#""@\""# {
            lit!("@\\")
        });
    }

    #[test]
    fn char_class_named() {
        test!(parse_char_class "[.]" {
            class![.]
        });
        test!(parse_char_class "[Test]" {
            class![Test]
        });
        test!(parse_char_class "[a b c]" {
            class!["\u{7}" b c]
        });
        test!(parse_char_class "[.]" {
            class![.]
        });
    }

    #[test]
    fn char_class_ranges() {
        test!(parse_char_class r#"['+#-"\']"# {
            class![r#"+#-"\"#]
        });
        test!(parse_char_class "['a']" {
            class!["a"]
        });
        test!(parse_char_class "['Test']" {
            class!["Test"]
        });
        test!(parse_char_class "[u+0]" {
            class!["\0"]
        });
        test!(parse_char_class r#"["!"-U+255]"# {
            class!['!'-'\u{255}']
        });
        test!(parse_char_class r#"[U+0000 - U+FFFF]"# {
            class!['\0'-'\u{FFFF}']
        });
    }

    #[test]
    fn boundary() {
        test!(parse_boundary "<%" {
            boundary!(<%)
        });
        test!(parse_boundary "%>" {
            boundary!(%>)
        });
        test!(parse_boundary "%" {
            boundary!(%)
        });
        test!(parse_boundary "not %" {
            boundary!(not %)
        });
    }

    #[test]
    fn sequence() {
        test!(parse_sequence "[.] % 'test' ['test']" {
            group!(
                class![.],
                boundary!(%),
                lit!("test"),
                class!["test"],
            )
        });
    }

    #[test]
    fn or() {
        test!(parse_or "[.] % | 'test' ['test'] | %" {
            alt![
                group!(class![.], boundary!(%)),
                group!(lit!("test"), class!["test"]),
                boundary!(%),
            ]
        });
    }

    #[test]
    fn group() {
        test!(parse_or "(([.]) % | 'test' (['test'] | %))" {
            alt![
                group!(class![.], boundary!(%)),
                group!(
                    lit!("test"),
                    alt![ class!["test"], boundary!(%) ],
                ),
            ]
        });
    }

    #[test]
    fn capturing_group() {
        test!(parse_or "(:([.]) % | 'test' :foo(['test'] | %))" {
            alt![
                group!(
                    group![:(class![.])],
                    boundary!(%),
                ),
                group!(
                    lit!("test"),
                    group![:foo( alt![ class!["test"], boundary!(%) ] )],
                ),
            ]
        });
    }

    #[test]
    fn fixes() {
        test!(parse_fixes "%*" {
            repeat!(boundary!(%), *)
        });
        test!(parse_fixes "%+" {
            repeat!(boundary!(%), +)
        });
        test!(parse_fixes "%?" {
            repeat!(boundary!(%), ?)
        });
        test!(parse_fixes "%{,}" {
            repeat!(boundary!(%), {0,})
        });

        test!(parse_fixes "%* greedy" {
            repeat!(boundary!(%), * greedy)
        });
        test!(parse_fixes "%+ greedy" {
            repeat!(boundary!(%), + greedy)
        });
        test!(parse_fixes "%? greedy" {
            repeat!(boundary!(%), ? greedy)
        });
        test!(parse_fixes "%{,} greedy" {
            repeat!(boundary!(%), {0,} greedy)
        });

        test!(parse_fixes "%{0,}" {
            repeat!(boundary!(%), {0,})
        });
        test!(parse_fixes "%{3,}" {
            repeat!(boundary!(%), {3,})
        });
        test!(parse_fixes "%{3,6}" {
            repeat!(boundary!(%), {3,6})
        });
        test!(parse_fixes "%{,6}" {
            repeat!(boundary!(%), {0,6})
        });

        test!(parse_fixes "%**" {
            repeat!(repeat!(boundary!(%), *), *)
        });
        test!(parse_fixes "%{3}{4}" {
            repeat!(repeat!(boundary!(%), {3,3}), {4,4})
        });
    }

    #[test]
    fn lookaround() {
        test!(parse_fixes "<< 'test'" {
            look! { << lit!("test") }
        });
        test!(parse_fixes "<< 'test'*" {
            look! { << repeat!(lit!("test"), {0,}) }
        });
        test!(parse_fixes "<< 'foo' 'bar'" {
            look! { << group!(lit!("foo"), lit!("bar")) }
        });
        test!(parse_fixes "<< 'test'* | [.]" {
            look! { << alt![
                repeat!(lit!("test"), {0,}),
                class![.]
            ] }
        });
    }
}
