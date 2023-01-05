#[doc(hidden)]
#[macro_export]
macro_rules! text_impl {
    // c:"text"
    ([$color_id:ident : $lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Color::$color_id), text: $lit, ticks: true
        })
    };
    // c:{expr}
    ([$color_id:ident : {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Color::$color_id), text: $ex, ticks: true
        })
    };
    // c!"text"
    ([$color_id:ident ! $lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Color::$color_id), text: $lit, ticks: false
        })
    };
    // c!{expr}
    ([$color_id:ident ! {$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment {
            style: Some($crate::Color::$color_id), text: $ex, ticks: false
        })
    };
    // "text"
    ([$lit:literal $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment::new($lit))
    };
    // {expr}
    ([{$ex:expr} $($rest:tt)*] $($done:tt)*) => {
        $crate::text_impl!([$($rest)*] $($done)*, $crate::Segment::new($ex))
    };
    ([], $($done:tt)*) => {
        &[$($done)*]
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sections_impl {
    // ["text"]
    (@[ [$($text:tt)*] $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Text($crate::text![ $($text)* ])
        )
    };
    // table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ table $mode:ident { $( $key:literal => { $($inner:tt)* } )* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Table($crate::TableMode::$mode, &[$(
                ( $key, $crate::sections!( $($inner)* ) ),
            )*])
        )
    };
    // "NAME" {...}
    (@[ $name:literal { $($inner:tt)* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::Name($name, $crate::sections!( $($inner)* ))
        )
    };
    // Short ["text"]
    (@[ $wrapper:ident [$($text:tt)* ] $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Text($crate::text![ $($text)* ])
            )
        )
    };
    // Short table {
    //     "foo" => {...}
    //     "bar" => {...}
    // }
    (@[ $wrapper:ident table $mode:ident { $( $key:literal => { $($inner:tt)* } )* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[ $($rest)* ] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Table($crate::TableMode::$mode, &[$(
                    ( $key, $crate::sections!( $($inner)* ) ),
                )*])
            )
        )
    };
    // Short "NAME" {...}
    (@[ $wrapper:ident $name:literal { $($inner:tt)* } $($rest:tt)* ] $($done:tt)*) => {
        $crate::sections_impl!(
            @[$($rest)*] $($done)*,
            $crate::HelpSection::$wrapper(
                &$crate::HelpSection::Name($name, $crate::sections!( $($inner)* ))
            )
        )
    };

    (@[], $($done:tt)*) => {
        &[ $($done)* ]
    };
}

/// Macro to declare a list of text segments. A segment can be written as
///
/// - `"text"` (a string literal)
/// - `{expr}` (an expression that evaluates to a string slice)
///
/// Each segment can be preceded by one of
///
/// - `c:`, where `c` is a [`Color`](crate::Color) variant; the segment is
///   printed in color if supported, otherwise it is wrapped in backticks
/// - `c!`, where `c` is a [`Color`](crate::Color) variant; the segment is
///   printed in color if supported, otherwise no formatting is applied
///
/// Each color can be abbreviated with its first letter (cyan ➔ c, green ➔ g,
/// magenta ➔ m, red ➔ r, yellow ➔ y); use an uppercase letter to make it
/// bold (bold cyan ➔ C, etc.)
///
/// Segments are _not_ separated with commas, for example:
///
/// ```
/// // "warning" is yellow and bold, "world" is cyan, or wrapped in backticks
/// let _segments = text!(Y!"warning" ": hello" c:"world");
///
/// // the value of the FOO environment variable is printed in magenta
/// let _segments = text!("FOO is " m!{env!("FOO")});
/// ```
#[macro_export]
macro_rules! text {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        $crate::text_impl!([ $($rest)* ])
    };
}

/// Macro to declare a list of help sections. This can be passed to
/// [`Help`](crate::Help) to print it.
///
/// There are three kinds of sections:
///
/// 1. Normal sections, wrapped in square brackets. Refer to the
///    [`text` macro][text] for the syntax. Example:
///
///    ```
///    ["test" c:"cyan" R!"bold red"]
///    ```
///
///    Each section is terminated by a line break.
///
/// 2. Named sections. Example:
///
///    ```
///    "USAGE" {
///        ["section 1"]
///        ["section 2"]
///    }
///    ```
///
///    Named sections are always preceded by a blank line. Child sections are
///    indented with 4 spaces.
///
/// 3. Tables. Example:
///
///    ```
///    table Auto {
///        "argument 1" => {
///            ["help for argument 1"]
///        }
///        "argument 2" => {
///            ["help for argument 2"]
///            ["and some more help!"]
///        }
///    }
///    ```
///
///    With short help, this is rendered as
///
///    ```text
///    argument 1   help for argument 1
///    argument 2   help for argument 2
///                 and some more help!
///    ```
///
///    With long help, this is rendered as
///
///    ```text
///    argument 1
///            help for argument 1
///
///    argument 2
///            help for argument 2
///            and some more help!
///    ```
///
///    The argument name (left column) must be a string literal. It is displayed
///    in color.
///
///    The `table` keyword must be followed by either `Auto` or `Compact`. If
///    `Compact` is used, then the compact format is used for both the short and
///    long help. If `Auto` is used, the compact format is used for short help
///    and the longer format is used for long help.
///
/// Beyond that, each section can be preceded by `Short` or `Long`. By default,
/// sections are included both in the long and short help. With the `Short`
/// modifier, it is _only_ shown in the short help, and sections preceded by
/// `Long` only appear in the long help. Example:
///
/// ```
/// sections!(
///     Short ["Short help text"]
///     Long ["This is more detailed help text"]
///     ["This is shown either way"]
///
///     table Auto {
///         "argument 1" => {
///             ["description"]
///             Long ["Further details only shown in long help"]
///         }
///         "argument 2" => {
///             Long ["This argument isn't shown in the short help"]
///         }
///     }
///
///     // table only shown in long help:
///     Long table Compact {}
///
///     Long "MORE DETAILS" {
///         ["named section only shown in long help"]
///     }
/// );
/// ```
#[macro_export]
macro_rules! sections {
    () => {
        &[]
    };
    ($($rest:tt)*) => {
        $crate::sections_impl!(@[ $($rest)* ])
    };
}
