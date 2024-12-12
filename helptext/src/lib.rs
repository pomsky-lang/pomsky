//! Create beautiful help messages.
//!
//! The syntax of the help message is explained in the [`sections`] macro.
//!
//! The syntax for colorizing text is explained in the [`text`] macro.
//!
//! ```
//! use helptext::{Help, sections};
//!
//! const HELP: Help = Help(sections!(
//!     ["my-cool-program " {env!("CARGO_PKG_VERSION")}]
//!     ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
//!     []
//!     "USAGE" {
//!         ["my-cool-program [OPTIONS] <INPUT>"]
//!     }
//!     "OPTIONS" {
//!         table Auto {
//!             "-h, --help" => {
//!                 ["Print help information"]
//!                 Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
//!             }
//!             "-p, --path <FILE>" => {
//!                 ["File containing the pomsky expression to compile"]
//!             }
//!             "-V, --version" => {
//!                 ["Print version information"]
//!             }
//!             "-W, --warnings <DIAGNOSTICS>" => {
//!                 Short ["Disable certain warnings (disable all with " c:"-W0" ")"]
//!                 Long ["Disable some or all warnings. A single warning can be disabled by specifying
//! the name followed by " c:"=0" ", for example:
//!
//!     " c!"-Wcompat=0" "
//!
//! Multiple warnings can be disabled by setting this option multiple times, or
//! using a comma-separated list:
//!
//!     " c!"-Wcompat=0 -Wdeprecated=0
//!     -Wcompat=0,deprecated=0" "
//!
//! To disable all warnings, use " c:"-W0" ".
//!
//! Currently, the following warnings can be disabled:"]
//!                 Long table Compact {
//!                     "compat"     => { ["Compatibility warnings"] }
//!                     "deprecated" => { ["A used feature will be removed in the future"] }
//!                 }
//!             }
//!         }
//!     }
//! ));
//!
//! fn print_short_help(use_colors: bool) {
//!     HELP.write(
//!         &mut std::io::stdout().lock(),
//!         false,  // don't show long help
//!         use_colors,
//!     );
//! }
//!
//! fn print_long_help(use_colors: bool) {
//!     HELP.write(
//!         &mut std::io::stdout().lock(),
//!         true,  // show long help
//!         use_colors,
//!     );
//! }
//! ```
//!
//! Result:
//!
//! ![Short help](https://raw.githubusercontent.com/pomsky-lang/pomsky/main/helptext/docs/short_help.png)
//!
//! ![Long help](https://raw.githubusercontent.com/pomsky-lang/pomsky/main/helptext/docs/long_help.png)

mod color;
mod help;
mod macros;

pub use color::Color;
pub use help::*;
