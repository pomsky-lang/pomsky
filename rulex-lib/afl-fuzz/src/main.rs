use arbitrary::{Arbitrary, Unstructured};
use rulex::Rulex;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let mut u = Unstructured::new(data);
        if let Ok((input, parse_options, compile_options)) = Arbitrary::arbitrary(&mut u) {
            let _ = Rulex::parse_and_compile(input, parse_options, compile_options);
        }
    });
}
