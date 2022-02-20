macro_rules! tokens {
    () => {
        $crate::parse::Tokens::empty()
    };
}

macro_rules! test {
    ($f:ident $lit:literal { $($rest:tt)* }) => {{
        let mut buf = Vec::new();
        assert_eq!(
            $f($crate::parse::Tokens::tokenize($lit, &mut buf)),
            Ok((tokens![], $($rest)* ))
        );
    }};
}

macro_rules! lit {
    ($lit:literal) => {
        $crate::Rulex::Literal($lit)
    };
}

macro_rules! class {
    ([ $lit:literal ]) => {
        $crate::Rulex::CharClass($crate::char_class::CharClass::from_chars($lit))
    };
    (<.>) => {
        $crate::Rulex::CharClass($crate::char_class::CharClass::from_named("."))
    };
    (<$i:ident>) => {
        $crate::Rulex::CharClass($crate::char_class::CharClass::from_named(stringify!($i)))
    };
    ($start:literal - $end:literal) => {
        $crate::Rulex::CharClass(
            $crate::char_class::CharClass::try_from_range($start, $end).unwrap(),
        )
    };
}

macro_rules! alt {
    ($($e:expr),* $(,)?) => {{
        $crate::alternation::Alternation::new_rulex(
            vec![$( $e ),*],
        )
    }};
}

macro_rules! group {
    (:$name:ident( $( $e:expr ),* $(,)? )) => {
        $crate::Rulex::Group($crate::group::Group::new(
            vec![$( $e ),*],
            Some($crate::group::Capture::new(Some(stringify!($name)))),
        ))
    };
    (:( $( $e:expr ),* $(,)? )) => {
        $crate::Rulex::Group($crate::group::Group::new(
            vec![$( $e ),*],
            Some($crate::group::Capture::new(None)),
        ))
    };
    ( $( $e:expr ),* $(,)? ) => {
        $crate::Rulex::Group($crate::group::Group::new(
            vec![$( $e ),*],
            None,
        ))
    };
}

macro_rules! boundary {
    (%-) => {
        $crate::Rulex::Boundary($crate::boundary::Boundary::Start)
    };
    (-%) => {
        $crate::Rulex::Boundary($crate::boundary::Boundary::End)
    };
    (%) => {
        $crate::Rulex::Boundary($crate::boundary::Boundary::Word)
    };
    (%!) => {
        $crate::Rulex::Boundary($crate::boundary::Boundary::NotWord)
    };
}

macro_rules! repeat {
    (~greedy) => {
        $crate::repetition::Greedy::Yes
    };
    (~) => {
        $crate::repetition::Greedy::No
    };

    ($e:expr, { $lower:literal, $upper:literal } $($greedy:ident)?) => {{
        $crate::Rulex::Repetition(Box::new($crate::repetition::Repetition::new(
            $e,
            ($lower, Some($upper)).try_into().unwrap(),
            repeat!(~$($greedy)?)
        )))
    }};

    ($e:expr, { $lower:literal, } $($greedy:ident)?) => {{
        $crate::Rulex::Repetition(Box::new($crate::repetition::Repetition::new(
            $e,
            ($lower, None).try_into().unwrap(),
            repeat!(~$($greedy)?)
        )))
    }};

    ($e:expr, ? $($greedy:ident)?) => {{
        $crate::Rulex::Repetition(Box::new($crate::repetition::Repetition::new(
            $e,
            $crate::repetition::RepetitionKind::zero_one(),
            repeat!(~$($greedy)?)
        )))
    }};
    ($e:expr, * $($greedy:ident)?) => {{
        $crate::Rulex::Repetition(Box::new($crate::repetition::Repetition::new(
            $e,
            $crate::repetition::RepetitionKind::zero_inf(),
            repeat!(~$($greedy)?)
        )))
    }};
    ($e:expr, + $($greedy:ident)?) => {{
        $crate::Rulex::Repetition(Box::new($crate::repetition::Repetition::new(
            $e,
            $crate::repetition::RepetitionKind::one_inf(),
            repeat!(~$($greedy)?)
        )))
    }};
}
