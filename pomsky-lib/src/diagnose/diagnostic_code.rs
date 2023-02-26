use std::fmt::{self, Write};

use pomsky_syntax::diagnose::{
    CharClassError, CharStringError, CodePointError, LexErrorMsg, ParseErrorKind, ParseWarningKind,
    RepetitionError,
};

use super::CompileErrorKind;

macro_rules! diagnostic_code {
    {
        $( #[$m:meta] )*
        $visib:vis enum $name:ident {
            $( $variant:ident = $num:literal, )*
        }
    } => {
        $( #[$m] )*
        $visib enum $name {
            $( $variant = $num, )*
        }

        impl TryFrom<u16> for $name {
            type Error = ();

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                Ok(match value {
                    $( $num => $name::$variant, )*
                    _ => return Err(()),
                })
            }
        }
    };
}

diagnostic_code! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u16)]
    #[non_exhaustive]
    #[allow(missing_docs)]
    pub enum DiagnosticCode {
        // Lex errors
        UnknownToken = 1,
        RegexGroupSyntax = 2,
        RegexBackslashSyntax = 3,
        UnclosedString = 4,
        DeprecatedToken = 5,

        // Parse errors
        UnexpectedToken = 100,
        UnexpectedReservedWord = 101,
        NonAsciiIdentAfterColon = 102,
        IdentTooLong = 103,
        RangeIsNotIncreasing = 104,
        DeprecatedSyntax = 105,
        UnallowedNot = 106,
        UnallowedMultiNot = 107,
        InvalidEscapeInString = 108,
        CodePointInvalid = 109,
        InvalidNumber = 110,
        RepetitionNotAscending = 111,
        RepetitionChain = 112,
        CharRangeStringEmpty = 113,
        CharRangeTooManyCodePoints = 114,
        CharClassHasDescendingRange = 115,
        CharClassUnknownShorthand = 116,
        CharClassIllegalNegation = 117,
        CharClassUnallowedCombination = 118,
        NegatedHorizVertSpace = 119,

        // Currently a parse error, but it should be a compile error
        LetBindingExists = 300,

        // Compile errors
        UnsupportedRegexFeature = 301,
        UnsupportedPomskySyntax = 302,
        HugeReference = 303,
        UnknownReference = 304,
        NameUsedMultipleTimes = 305,
        EmptyClass = 306,
        EmptyClassNegated = 307,
        CaptureInLet = 308,
        ReferenceInLet = 309,
        UnknownVariable = 310,
        RecursiveVariable = 311,
        RangeIsTooBig = 312,
        RecursionLimit = 313,

        // Warning indicating something might not be supported
        PossiblyUnsupported = 400,
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f.write_char('P');
        f.write_fmt(format_args!("{:04}", *self as u16))
    }
}

impl From<LexErrorMsg> for DiagnosticCode {
    fn from(value: LexErrorMsg) -> Self {
        use LexErrorMsg as M;
        match value {
            M::GroupNonCapturing
            | M::GroupLookahead
            | M::GroupLookaheadNeg
            | M::GroupLookbehind
            | M::GroupLookbehindNeg
            | M::GroupNamedCapture
            | M::GroupPcreBackreference
            | M::GroupComment
            | M::GroupAtomic
            | M::GroupConditional
            | M::GroupBranchReset
            | M::GroupSubroutineCall
            | M::GroupOther => DiagnosticCode::RegexGroupSyntax,
            M::Backslash
            | M::BackslashU4
            | M::BackslashX2
            | M::BackslashUnicode
            | M::BackslashProperty
            | M::BackslashGK => DiagnosticCode::RegexBackslashSyntax,
            M::UnclosedString => DiagnosticCode::UnclosedString,
            M::DeprStart | M::DeprEnd => DiagnosticCode::DeprecatedToken,
            _ => panic!("Unhandled lexer error message {value:?}"),
        }
    }
}

impl<'a> From<&'a CharClassError> for DiagnosticCode {
    fn from(value: &'a CharClassError) -> Self {
        use CharClassError as E;
        match value {
            E::Empty => Self::EmptyClass,
            E::CaretInGroup | E::Invalid => Self::UnexpectedToken,
            E::DescendingRange(_, _) => Self::CharClassHasDescendingRange,
            E::Unallowed => Self::CharClassUnallowedCombination,
            E::UnknownNamedClass { .. } => Self::CharClassUnknownShorthand,
            E::Negative => Self::CharClassIllegalNegation,
            _ => panic!("Unhandled char class error message {value:?}"),
        }
    }
}

impl<'a> From<&'a ParseErrorKind> for DiagnosticCode {
    fn from(value: &'a ParseErrorKind) -> Self {
        use ParseErrorKind as P;
        use RepetitionError as R;
        match value {
            P::UnknownToken => Self::UnknownToken,
            &P::LexErrorWithMessage(msg) => msg.into(),
            P::KeywordAfterLet(_) | P::KeywordAfterColon(_) | P::UnexpectedKeyword(_) => {
                Self::UnexpectedReservedWord
            }
            P::NonAsciiIdentAfterColon(_) => Self::NonAsciiIdentAfterColon,
            P::GroupNameTooLong(_) => Self::IdentTooLong,
            P::Deprecated(_) => Self::DeprecatedSyntax,
            P::Expected(_)
            | P::LeftoverTokens
            | P::ExpectedToken(_)
            | P::LonePipe
            | P::CodePointAfterLet(_) => Self::UnexpectedToken,
            P::RangeIsNotIncreasing => Self::RangeIsNotIncreasing,
            P::UnallowedNot => Self::UnallowedNot,
            P::UnallowedMultiNot(_) => Self::UnallowedMultiNot,
            P::LetBindingExists => Self::LetBindingExists,
            P::InvalidEscapeInStringAt(_) => Self::InvalidEscapeInString,
            P::CharString(CharStringError::Empty) => Self::CharRangeStringEmpty,
            P::CharString(CharStringError::TooManyCodePoints) => Self::CharRangeTooManyCodePoints,
            P::CharClass(e) => e.into(),
            P::CodePoint(CodePointError::Invalid) => Self::CodePointInvalid,
            P::Number(_) => Self::InvalidNumber,
            P::Repetition(R::NotAscending) => Self::RepetitionNotAscending,
            P::Repetition(R::Multi | R::QmSuffix) => Self::RepetitionChain,
            P::RecursionLimit => Self::RecursionLimit,
            _ => panic!("Unhandled parser error message {value:?}"),
        }
    }
}

impl<'a> From<&'a CompileErrorKind> for DiagnosticCode {
    fn from(value: &'a CompileErrorKind) -> Self {
        use CompileErrorKind as C;
        match value {
            C::ParseError(p) => p.into(),
            C::Unsupported(..) => Self::UnsupportedRegexFeature,
            C::UnsupportedPomskySyntax(_) => Self::UnsupportedPomskySyntax,
            C::HugeReference => Self::HugeReference,
            C::UnknownReferenceNumber(_) | C::UnknownReferenceName { .. } => Self::UnknownReference,
            C::NameUsedMultipleTimes(_) => Self::NameUsedMultipleTimes,
            C::EmptyClass => Self::EmptyClass,
            C::EmptyClassNegated { .. } => Self::EmptyClassNegated,
            C::CaptureInLet => Self::CaptureInLet,
            C::ReferenceInLet => Self::ReferenceInLet,
            C::RelativeRefZero => Self::UnknownReference,
            C::UnknownVariable { .. } => Self::UnknownVariable,
            C::RecursiveVariable => Self::RecursiveVariable,
            C::RangeIsTooBig(_) => Self::RangeIsTooBig,
            C::NegatedHorizVertSpace => Self::NegatedHorizVertSpace,
        }
    }
}

impl<'a> From<&'a ParseWarningKind> for DiagnosticCode {
    fn from(value: &'a ParseWarningKind) -> Self {
        match value {
            ParseWarningKind::Deprecation(_) => Self::DeprecatedSyntax,
        }
    }
}
