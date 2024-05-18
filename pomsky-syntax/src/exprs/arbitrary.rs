use arbitrary::{Arbitrary, Unstructured};

pub(crate) struct Ident(pub(crate) String);

impl Ident {
    pub(crate) fn create(u: &mut Unstructured<'_>) -> Result<String, arbitrary::Error> {
        Ok(Ident::arbitrary(u)?.0)
    }
}

impl Arbitrary<'_> for Ident {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let options = [
            "foo", "bar", "baz", "quux", "blabla", "hello", "world", "match", "regular", "based",
            "a", "b", "c", "d", "e", "f", "g",
        ];
        let idx = u.int_in_range(0..=options.len() as u8 - 1)?;
        let name = options[idx as usize];
        Ok(Ident(name.to_string()))
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

#[allow(unused)]
pub(crate) struct Digits(pub(crate) Box<[u8]>);

impl Digits {
    pub(crate) fn create(
        u: &mut Unstructured<'_>,
        radix: u8,
    ) -> Result<Box<[u8]>, arbitrary::Error> {
        let len = u.arbitrary_len::<u8>()?.min(10);
        let mut digits = Vec::with_capacity(len);
        for _ in 0..len {
            digits.push(u.int_in_range(0..=radix - 1)?);
        }
        Ok(digits.into_boxed_slice())
    }
}

impl Arbitrary<'_> for Digits {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let len = u.arbitrary_len::<u8>()?.min(10);
        let mut digits = Vec::with_capacity(len);
        for _ in 0..len {
            digits.push(u.int_in_range(0..=9)?);
        }
        Ok(Digits(digits.into_boxed_slice()))
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (0, Some(10))
    }
}
