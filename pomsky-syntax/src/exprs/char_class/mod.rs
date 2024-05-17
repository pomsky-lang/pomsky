//! Implements _character classes_. The analogue in the regex world are
//! [character classes](https://www.regular-expressions.info/charclass.html),
//! [shorthand character classes](https://www.regular-expressions.info/shorthand.html),
//! [non-printable characters](https://www.regular-expressions.info/nonprint.html),
//! [Unicode categories/scripts/blocks](https://www.regular-expressions.info/unicode.html#category),
//! [POSIX classes](https://www.regular-expressions.info/posixbrackets.html#class) and the
//! [dot](https://www.regular-expressions.info/dot.html).
//!
//! All kinds of character classes mentioned above require `[` square brackets
//! `]` in Pomsky. A character class can be negated by putting the keyword `not`
//! after the opening bracket. For example, `![.]` compiles to `\n`.
//!
//! ## Items
//!
//! A character class can contain multiple _items_, which can be
//!
//! - A __code point__, e.g. `['a']` or `[U+107]`
//!
//!   - This includes [non-printable characters](https://www.regular-expressions.info/nonprint.html).\
//!     Supported are `[n]`, `[r]`, `[t]`, `[a]`, `[e]` and `[f]`.
//!
//! - A __range of code points__. For example, `[U+10 - U+200]` matches any code
//!   point P where `U+10 ≤ P ≤ U+200`
//!
//! - A __named character class__, which can be one of
//!
//!   - a [shorthand character class](https://www.regular-expressions.info/shorthand.html).\
//!     Supported are `[w]`, `[d]`, `[s]`, `[h]`, `[v]` and `[R]`.
//!
//!   - a [POSIX class](https://www.regular-expressions.info/posixbrackets.html#class).\
//!     Supported are `[ascii_alnum]`, `[ascii_alpha]`, `[ascii]`,
//!     `[ascii_blank]`, `[ascii_cntrl]`, `[ascii_digit]`, `[ascii_graph]`,
//!     `[ascii_lower]`, `[ascii_print]`, `[ascii_punct]`, ´ `[ascii_space]`,
//!     `[ascii_upper]`, `[ascii_word]` and `[ascii_xdigit]`.\ _Note_: POSIX
//!     classes are not Unicode aware!\ _Note_: They're converted to ranges,
//!     e.g. `[ascii_alpha]` = `[a-zA-Z]`.
//!
//!   - a [Unicode category, script or block](https://www.regular-expressions.info/unicode.html#category).\
//!     For example: `[Letter]` compiles to `\p{Letter}`. Pomsky currently
//!     treats any uppercase identifier except `R` as Unicode class.
//!
//! ## Compilation
//!
//! When a character class contains only a single item (e.g. `[w]`), the
//! character class is "flattened":
//!
//! - `['a']` = `a`
//! - `[w]` = `\w`
//! - `[Letter]` = `\p{Letter}`
//!
//! When there is more than one item or a range (e.g. `['a'-'z' '!']`), a regex
//! character class is created:
//!
//! - `['a'-'z' '!']` = `[a-z!]`
//! - `[w e Punctuation]` = `[\w\e\p{Punctuation}]`
//!
//! ### Negation
//!
//! Negation is implemented as follows:
//!
//! - Ranges and chars such as `!['a'-'z' '!' e]` are wrapped in a negative
//!   character class, e.g. `[^a-z!\e]`.
//!
//! - The `h`, `v` and `R` shorthands are also wrapped in a negative character
//!   class.
//!
//! - The `w`, `d` and `s` shorthands are negated by making them uppercase
//!   (`![w]` = `\W`), except when there is more than one item in the class
//!   (`![w '-']` = `[^\w\-]`)
//!
//! - `w`, `s`, `d` and Unicode categories/scripts/blocks can be negated
//!   individually _within a character class_, e.g. `[s !s]` = `[\s\S]`,
//!   `![!Latin 'a']` = `[^\P{Latin}a]`.
//!
//!   When a negated character class only contains 1 item, which is also
//! negated, the class is   removed and the negations cancel each other out:
//! `![!w]` = `\w`, `![!L]` = `\p{L}`.

use crate::Span;

pub use char_group::{CharGroup, GroupItem, GroupName};
pub use unicode::{Category, CodeBlock, OtherProperties, Script};

mod ascii;
pub(crate) mod char_group;
pub(crate) mod unicode;

pub use unicode::{blocks_supported_in_dotnet, list_shorthands};

/// A _character class_. Refer to the [module-level documentation](self) for
/// details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharClass {
    pub inner: Vec<GroupItem>,
    pub span: Span,
}

impl CharClass {
    pub fn new(inner: Vec<GroupItem>, span: Span) -> Self {
        CharClass { inner, span }
    }

    #[cfg(feature = "dbg")]
    pub(super) fn pretty_print(&self, buf: &mut crate::PrettyPrinter) {
        buf.push('[');
        for (i, item) in self.inner.iter().enumerate() {
            if i > 0 {
                buf.push(' ');
            }
            item.pretty_print(buf);
        }
        buf.push(']');
    }
}
