/// A trait to construct simple regex-like automatons that can't backtrack.
pub(crate) trait MicroRegex {
    type Context;

    /// If the micro regex matches at the beginning of the haystack, return the length of the match.
    fn is_start(&self, haystack: &str) -> Option<(usize, Self::Context)>;

    fn split_off<'a>(&self, haystack: &'a str) -> Option<(&'a str, &'a str)> {
        let (len, _context) = self.is_start(haystack)?;
        Some(haystack.split_at(len))
    }

    fn ctx<C>(self, ctx: C) -> Ctx<Self, C>
    where
        Self: Sized,
    {
        Ctx(self, ctx)
    }
}

impl MicroRegex for str {
    type Context = ();

    fn is_start(&self, haystack: &str) -> Option<(usize, ())> {
        if haystack.starts_with(self) {
            Some((self.len(), ()))
        } else {
            None
        }
    }
}

impl MicroRegex for char {
    type Context = ();

    fn is_start(&self, haystack: &str) -> Option<(usize, ())> {
        if haystack.starts_with(*self) {
            Some((self.len_utf8(), ()))
        } else {
            None
        }
    }
}

impl<T: MicroRegex> MicroRegex for [T] {
    type Context = T::Context;

    fn is_start(&self, haystack: &str) -> Option<(usize, Self::Context)> {
        self.iter().find_map(|mr| mr.is_start(haystack))
    }
}

impl<T: MicroRegex + ?Sized> MicroRegex for &'_ T {
    type Context = T::Context;

    fn is_start(&self, haystack: &str) -> Option<(usize, Self::Context)> {
        (*self).is_start(haystack)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CharIs<I: Fn(char) -> bool>(pub(crate) I);

impl<I: Fn(char) -> bool> MicroRegex for CharIs<I> {
    type Context = ();

    fn is_start(&self, haystack: &str) -> Option<(usize, ())> {
        haystack.chars().next().filter(|&c| self.0(c)).map(|c| (c.len_utf8(), ()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Many0<I>(pub(crate) I);

impl<I: MicroRegex> MicroRegex for Many0<I> {
    type Context = ();

    fn is_start(&self, mut haystack: &str) -> Option<(usize, ())> {
        let mut len = 0;
        while let Some((add, _)) = self.0.is_start(haystack) {
            len += add;
            haystack = &haystack[add..];
        }
        Some((len, ()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Many1<I>(pub(crate) I);

impl<I: MicroRegex> MicroRegex for Many1<I> {
    type Context = ();

    fn is_start(&self, mut haystack: &str) -> Option<(usize, ())> {
        let (mut len, _) = self.0.is_start(haystack)?;
        haystack = &haystack[len..];
        while let Some((add, _)) = self.0.is_start(haystack) {
            len += add;
            haystack = &haystack[add..];
        }
        Some((len, ()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Ctx<I, C>(pub(crate) I, pub(crate) C);

impl<C: Clone, I: MicroRegex> MicroRegex for Ctx<I, C> {
    type Context = C;

    fn is_start(&self, haystack: &str) -> Option<(usize, C)> {
        let (len, _) = self.0.is_start(haystack)?;
        Some((len, self.1.clone()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Capture<I>(pub(crate) I);

macro_rules! impl_tuple {
    ($head_t:ident) => {
        impl_tuple!(~single: $head_t);
    };
    ($head_t:ident $($t:ident)*) => {
        impl_tuple!(~single: $head_t $($t)*);
        impl_tuple!($($t)*);
    };

    (~single: $($v:ident)*) => {
        #[allow(unused_variables, non_camel_case_types)]
        impl<$( $v : MicroRegex ),*> MicroRegex for ($( $v, )*) {
            type Context = ();

            fn is_start(&self, haystack: &str) -> Option<(usize, ())> {
                let ($( $v, )*) = self;

                $(
                    let ($v, _) = $v.is_start(haystack)?;
                    let (_, haystack) = haystack.split_at($v);
                )*

                Some(($( $v + )* 0, ()))
            }
        }

        #[allow(unused_variables, non_camel_case_types)]
        impl<$( $v : MicroRegex ),*> MicroRegex for Capture<($( $v, )*)> {
            type Context = ($( $v::Context, )*);

            fn is_start(&self, haystack: &str) -> Option<(usize, Self::Context)> {
                let Capture(($( $v, )*)) = self;

                $(
                    let $v = $v.is_start(haystack)?;
                    let (_, haystack) = haystack.split_at($v.0);
                )*

                Some(($( $v.0 + )* 0, ($( $v.1, )*)))
            }
        }
    };
}

impl_tuple!(a b c d e f g h i j k l);
