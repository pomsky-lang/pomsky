use arbitrary::{Arbitrary, Unstructured};
use pomsky::Expr;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, parse_options, compile_options)) = Arbitrary::arbitrary(&mut u) {
            let _ = Expr::parse_and_compile(input, parse_options, compile_options);
        }
    });
}
