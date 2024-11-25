use std::ops::{Deref, DerefMut};

pub(crate) struct Deferred<'a, S, F: FnMut(&mut S)> {
    state: &'a mut S,
    mutate: F,
}

impl<'a, S, F: FnMut(&mut S)> Deferred<'a, S, F> {
    pub(crate) fn new(state: &'a mut S, mutate: F) -> Self {
        Deferred { state, mutate }
    }
}

impl<S, F: FnMut(&mut S)> Drop for Deferred<'_, S, F> {
    fn drop(&mut self) {
        let mutator = &mut self.mutate;
        mutator(self.state);
    }
}

impl<S, F: FnMut(&mut S)> Deref for Deferred<'_, S, F> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<S, F: FnMut(&mut S)> DerefMut for Deferred<'_, S, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

#[doc(hidden)]
pub(crate) trait Mutable {
    fn mutable(&mut self) -> &mut Self;
}

impl<T> Mutable for T {
    fn mutable(&mut self) -> &mut Self {
        self
    }
}

macro_rules! revert_on_drop {
    ($state:ident . $($id:tt).*) => {
        let __prev = $state.$($id).*;
        let mut $state = {
            use $crate::defer::Mutable as _;
            $crate::defer::Deferred::new($state.mutable(), move |$state| {
                $state.$($id).* = __prev;
            })
        };
    };
}
