use std::collections::HashSet;

use intersection::Intersection;

use crate::{
    Span,
    diagnose::{
        CharClassError, CharStringError, DeprecationWarning, NumberError, ParseWarningKind,
        RepetitionError,
    },
    error::{ParseError, ParseErrorKind as PEK},
    exprs::{negation::Negation, test::*, *},
    lexer::Token,
};

use super::{Parser, helper};

type PResult<T> = Result<T, ParseError>;

const MAX_GROUP_NUMBER: u32 = 65_535;

impl<'i> Parser<'i> {
    pub(super) fn parse_modified(&mut self) -> PResult<Rule> {
        let mut stmts = Vec::new();

        let was_lazy = self.is_lazy;
        let was_unicode_aware = self.is_unicode_aware;

        loop {
            let Some(stmt) = self
                .parse_mode_modifier()?
                .try_or_else(|| self.parse_let())?
                .try_or_else(|| self.parse_test())?
            else {
                break;
            };

            match &stmt.0 {
                Stmt::Enable(BooleanSetting::Lazy, _) => self.is_lazy = true,
                Stmt::Enable(BooleanSetting::Unicode, _) => self.is_unicode_aware = true,
                Stmt::Disable(BooleanSetting::Lazy, _) => self.is_lazy = false,
                Stmt::Disable(BooleanSetting::Unicode, _) => self.is_unicode_aware = false,
                _ => {}
            }

            stmts.push(stmt);
        }

        self.recursion_start()?;
        let mut rule = self.parse_or()?;
        self.recursion_end();

        self.is_lazy = was_lazy;
        self.is_unicode_aware = was_unicode_aware;

        // TODO: This should not be part of the parser
        if stmts.len() > 1 {
            let mut set = HashSet::new();
            for (stmt, _) in &stmts {
                if let Stmt::Let(l) = stmt {
                    if set.contains(l.name()) {
                        return Err(PEK::LetBindingExists.at(l.name_span));
                    }
                    set.insert(l.name());
                }
            }
        }

        let span_end = rule.span();
        for (stmt, span) in stmts.into_iter().rev() {
            rule = Rule::StmtExpr(Box::new(StmtExpr::new(stmt, rule, span.join(span_end))));
        }

        Ok(rule)
    }

    fn parse_mode_modifier(&mut self) -> PResult<Option<(Stmt, Span)>> {
        let mode = if self.consume_reserved("enable") {
            true
        } else if self.consume_reserved("disable") {
            false
        } else {
            return Ok(None);
        };

        let span_start = self.last_span();
        let setting = if self.consume_reserved("lazy") {
            BooleanSetting::Lazy
        } else if self.consume_contextual_keyword("unicode") {
            BooleanSetting::Unicode
        } else {
            return Err(PEK::Expected("`lazy` or `unicode`").at(self.span()));
        };
        self.expect(Token::Semicolon)?;
        let span_end = self.last_span();
        let span = span_start.join(span_end);

        let stmt = if mode { Stmt::Enable(setting, span) } else { Stmt::Disable(setting, span) };

        Ok(Some((stmt, span)))
    }

    fn parse_let(&mut self) -> PResult<Option<(Stmt, Span)>> {
        if self.consume_reserved("let") {
            let span_start = self.last_span();
            let name_span = self.span();
            let name = self.expect_as(Token::Identifier).map_err(|e| {
                if self.is(Token::ReservedName) {
                    PEK::KeywordAfterLet(self.source_at(self.span()).to_owned()).at(e.span)
                } else {
                    e
                }
            })?;

            self.expect(Token::Equals)?;

            self.recursion_start()?;
            let rule = self.parse_or()?;
            self.recursion_end();

            self.expect(Token::Semicolon)
                .map_err(|p| PEK::Expected("expression or `;`").at(p.span))?;
            let span_end = self.last_span();

            Ok(Some((Stmt::Let(Let::new(name, rule, name_span)), span_start.join(span_end))))
        } else {
            Ok(None)
        }
    }

    fn parse_test(&mut self) -> PResult<Option<(Stmt, Span)>> {
        if self.consume_reserved("test") {
            let span_start = self.last_span();
            self.expect(Token::OpenBrace)?;

            let mut cases = Vec::new();
            while let Some(case) = self.parse_test_cases()? {
                cases.push(case);
            }

            self.expect(Token::CloseBrace)?;
            let span_end = self.last_span();
            let span = span_start.join(span_end);

            Ok(Some((Stmt::Test(Test { cases, span }), span)))
        } else {
            Ok(None)
        }
    }

    fn parse_test_cases(&mut self) -> PResult<Option<TestCase>> {
        if self.consume_contextual_keyword("match") {
            let mut matches = Vec::new();
            let mut literal = None;

            if self.peek() != Some((Token::Identifier, "in")) {
                matches.push(self.parse_test_match()?);
                while self.consume(Token::Comma) {
                    matches.push(self.parse_test_match()?);
                }
            }

            if self.consume_contextual_keyword("in") {
                literal = self.parse_literal()?;
                if literal.is_none() {
                    return Err(PEK::ExpectedToken(Token::String).at(self.span()));
                };
            }
            self.expect(Token::Semicolon)?;

            if let Some(literal) = literal {
                Ok(Some(TestCase::MatchAll(TestCaseMatchAll { literal, matches })))
            } else if matches.len() > 1 {
                let span = matches[0].span.join(matches.last().unwrap().span);
                Err(PEK::MultipleStringsInTestCase.at(span))
            } else {
                let match_ = matches.pop().unwrap();
                Ok(Some(TestCase::Match(match_)))
            }
        } else if self.consume_contextual_keyword("reject") {
            let as_substring = self.consume_contextual_keyword("in");

            let Some(literal) = self.parse_literal()? else {
                return Err(PEK::ExpectedToken(Token::String).at(self.span()));
            };

            self.expect(Token::Semicolon)?;

            Ok(Some(TestCase::Reject(TestCaseReject { literal, as_substring })))
        } else {
            Ok(None)
        }
    }

    fn parse_test_match(&mut self) -> PResult<TestCaseMatch> {
        let Some(literal) = self.parse_literal()? else {
            return Err(PEK::ExpectedToken(Token::String).at(self.span()));
        };
        let span_start = self.last_span();

        let mut captures = Vec::new();

        if self.consume_contextual_keyword("as") {
            self.expect(Token::OpenBrace)?;

            let mut is_first = true;
            loop {
                if !is_first && !self.consume(Token::Comma) {
                    break;
                }
                let Some(capture) = self.parse_test_capture()? else {
                    break;
                };
                captures.push(capture);
                is_first = false;
            }

            self.expect(Token::CloseBrace)?;
        }

        let span_end = self.last_span();
        Ok(TestCaseMatch { literal, captures, span: span_start.join(span_end) })
    }

    fn parse_test_capture(&mut self) -> PResult<Option<TestCapture>> {
        let ident = if let Some(n) = self.consume_number(u16::MAX)? {
            CaptureIdent::Index(n)
        } else if let Some(name) = self.consume_as(Token::Identifier) {
            CaptureIdent::Name(name.to_string())
        } else {
            return Ok(None);
        };
        let ident_span = self.last_span();

        self.expect(Token::Colon)?;
        let Some(literal) = self.parse_literal()? else {
            return Err(PEK::ExpectedToken(Token::String).at(self.span()));
        };
        Ok(Some(TestCapture { ident, ident_span, literal }))
    }

    fn parse_or(&mut self) -> PResult<Rule> {
        let mut span = self.span();
        let leading_pipe = self.consume(Token::Pipe);

        let mut alts = Vec::new();
        let Some(first_alt) = self.parse_and()? else {
            if leading_pipe {
                return Err(PEK::LonePipe.at(span));
            } else {
                return Ok(Rule::Literal(Literal::new("".to_string(), Span::default())));
            }
        };
        alts.push(first_alt);

        while self.consume(Token::Pipe) {
            let Some(next_alt) = self.parse_and()? else {
                return Err(PEK::LonePipe.at(self.last_span()));
            };
            span = span.join(next_alt.span());
            alts.push(next_alt);
        }

        if alts.len() == 1 {
            Ok(alts.pop().unwrap())
        } else {
            Ok(Rule::Alternation(Alternation { rules: alts, span }))
        }
    }

    fn parse_and(&mut self) -> PResult<Option<Rule>> {
        let span_start = self.span();
        let has_leading_ampersand = self.consume(Token::Ampersand);

        let Some(first_sequence) = self.parse_sequence()? else {
            if has_leading_ampersand {
                return Err(PEK::Expected("expression").at(self.span()));
            }
            return Ok(None);
        };
        if !self.is(Token::Ampersand) {
            return Ok(Some(first_sequence));
        }

        let mut rules = Vec::with_capacity(2);
        rules.push(first_sequence);
        loop {
            if !self.consume(Token::Ampersand) {
                let span = span_start.join(self.last_span());
                return Ok(Some(Rule::Intersection(Intersection { rules, span })));
            }

            let Some(next_sequence) = self.parse_sequence()? else {
                return Err(PEK::Expected("expression").at(self.span()));
            };
            rules.push(next_sequence);
        }
    }

    fn parse_sequence(&mut self) -> PResult<Option<Rule>> {
        let mut fixes = Vec::new();
        while let Some(fix) = self.parse_fixes()? {
            fixes.push(fix);
        }

        Ok(if fixes.is_empty() {
            None
        } else if fixes.len() == 1 {
            Some(fixes.pop().unwrap())
        } else {
            let start = fixes.first().map(Rule::span).unwrap_or_default();
            let end = fixes.last().map(Rule::span).unwrap_or_default();
            let span = start.join(end);

            Some(Rule::Group(Group::new(fixes, GroupKind::Implicit, span)))
        })
    }

    fn parse_fixes(&mut self) -> PResult<Option<Rule>> {
        let mut nots_span = self.span();
        let mut nots = 0usize;
        while self.consume(Token::Not) {
            nots += 1;
            nots_span = nots_span.join(self.last_span());
        }

        let Some(mut rule) = self.parse_lookaround()?.try_or_else(|| self.parse_atom())? else {
            if nots == 0 {
                return Ok(None);
            } else {
                return Err(PEK::Expected("expression").at(self.span()));
            }
        };

        for _ in 0..nots {
            rule = Rule::Negation(Box::new(Negation { rule, not_span: nots_span }));
        }

        while let Some((kind, quantifier, span)) = self.parse_repetition()? {
            let span = rule.span().join(span);
            rule = Rule::Repetition(Box::new(Repetition::new(rule, kind, quantifier, span)));
        }

        Ok(Some(rule))
    }

    fn parse_lookaround(&mut self) -> PResult<Option<Rule>> {
        let kind = if self.consume(Token::LookAhead) {
            LookaroundKind::Ahead
        } else if self.consume(Token::LookBehind) {
            LookaroundKind::Behind
        } else {
            return Ok(None);
        };
        let start_span = self.last_span();

        self.recursion_start()?;
        let rule = self.parse_modified()?;
        self.recursion_end();

        let span = rule.span();
        Ok(Some(Rule::Lookaround(Box::new(Lookaround::new(rule, kind, start_span.join(span))))))
    }

    /// Parse a repetition that can follow an atom: `+`, `?`, `*`, `{x}`,
    /// `{x,}`, `{,x}` or `{x,y}` optionally followed by the `greedy` or
    /// `lazy` keyword. `x` and `y` are number literals.
    fn parse_repetition(&mut self) -> PResult<Option<(RepetitionKind, Quantifier, Span)>> {
        let start = self.span();

        let kind = if self.consume(Token::Plus) {
            RepetitionKind::one_inf()
        } else if self.consume(Token::Star) {
            RepetitionKind::zero_inf()
        } else if self.consume(Token::QuestionMark) {
            RepetitionKind::zero_one()
        } else if let Some(kind) = self.parse_repetition_braces()? {
            kind
        } else {
            return Ok(None);
        };

        let quantifier = if self.consume_reserved("greedy") {
            Quantifier::Greedy
        } else if self.consume_reserved("lazy") {
            Quantifier::Lazy
        } else if self.is_lazy {
            Quantifier::DefaultLazy
        } else {
            Quantifier::DefaultGreedy
        };

        let multi_span = self.span();
        if self.consume(Token::Plus) || self.consume(Token::Star) {
            return Err(PEK::Repetition(RepetitionError::Multi).at(multi_span));
        } else if self.consume(Token::QuestionMark) {
            return Err(PEK::Repetition(RepetitionError::QmSuffix).at(multi_span));
        } else if self.parse_repetition_braces()?.is_some() {
            return Err(
                PEK::Repetition(RepetitionError::Multi).at(multi_span.join(self.last_span()))
            );
        }

        let end = self.last_span();
        Ok(Some((kind, quantifier, start.join(end))))
    }

    /// Parse `{2}`, `{2,}`, `{,2}` or `{2,5}`.
    fn parse_repetition_braces(&mut self) -> PResult<Option<RepetitionKind>> {
        if self.consume(Token::OpenBrace) {
            let num_start = self.span();

            // Both numbers and the comma are parsed optionally, then we check that one
            // of the allowed syntaxes is used: There must be at least one number, and if
            // there are two numbers, the comma is required. It also checks that the
            // numbers are in increasing order.
            let lower = self.consume_number(65_535)?;
            let comma = self.consume(Token::Comma);
            let upper = self.consume_number(65_535)?;

            let num_end = self.last_span();
            let num_span = num_start.join(num_end);

            let kind = match (lower, comma, upper) {
                (lower, true, upper) => (lower.unwrap_or(0), upper)
                    .try_into()
                    .map_err(|e| PEK::Repetition(e).at(num_span))?,

                (Some(_), false, Some(_)) => return Err(PEK::Expected("`}` or `,`").at(num_end)),
                (Some(rep), false, None) | (None, false, Some(rep)) => RepetitionKind::fixed(rep),
                (None, false, None) => return Err(PEK::Expected("number").at(self.span())),
            };

            self.expect(Token::CloseBrace)?;

            Ok(Some(kind))
        } else {
            Ok(None)
        }
    }

    fn parse_atom(&mut self) -> PResult<Option<Rule>> {
        Ok(self
            .parse_group()?
            .try_or_else(|| self.parse_string())?
            .try_or_else(|| self.parse_char_set())?
            .or_else(|| self.parse_boundary())
            .try_or_else(|| self.parse_reference())?
            .try_or_else(|| self.parse_code_point_rule())?
            .try_or_else(|| self.parse_range())?
            .try_or_else(|| self.parse_regex())?
            .try_or_else(|| self.parse_variable())?
            .or_else(|| self.parse_dot())
            .or_else(|| self.parse_recursion()))
    }

    /// Parses a (possibly capturing) group, e.g. `(E E | E)` or `:name(E)`.
    fn parse_group(&mut self) -> PResult<Option<Rule>> {
        let (kind, start_span) = self.parse_group_kind()?;
        if !kind.is_normal() {
            self.expect(Token::OpenParen)?;
        } else if !self.consume(Token::OpenParen) {
            return Ok(None);
        }

        self.recursion_start()?;
        let rule = self.parse_modified()?;
        self.recursion_end();

        self.expect(Token::CloseParen)
            .map_err(|p| PEK::Expected("`)` or an expression").at(p.span))?;
        // start_span may be 0..0, so we need to use join_unchecked
        let span = start_span.join_unchecked(self.last_span());

        let rule = Rule::Group(Group::new(vec![rule], kind, span));
        Ok(Some(rule))
    }

    /// Parses `:name` or just `:`. Returns the span of the colon with the name.
    fn parse_group_kind(&mut self) -> PResult<(GroupKind, Span)> {
        if self.consume_reserved("atomic") {
            let span = self.last_span();
            Ok((GroupKind::Atomic, span))
        } else if self.consume(Token::Colon) {
            let span = self.last_span();

            if let Some(keyword) = self.consume_as(Token::ReservedName) {
                return Err(PEK::KeywordAfterColon(keyword.into()).at(self.last_span()));
            }

            let name = self.consume_as(Token::Identifier);
            if let Some(name) = name {
                if let Some(invalid_index) = name.find(|c: char| !c.is_ascii_alphanumeric()) {
                    let c = name[invalid_index..].chars().next().unwrap();
                    let start = self.last_span().range_unchecked().start + invalid_index;
                    let len = c.len_utf8();
                    return Err(PEK::NonAsciiIdentAfterColon(c).at(Span::new(start, start + len)));
                }

                if name.len() > 32 {
                    return Err(PEK::GroupNameTooLong(name.len()).at(self.last_span()));
                }
            }
            Ok((GroupKind::Capturing(Capture::new(name)), span))
        } else {
            Ok((GroupKind::Normal, self.span()))
        }
    }

    /// Parses a string literal.
    fn parse_string(&mut self) -> PResult<Option<Rule>> {
        Ok(self.parse_literal()?.map(Rule::Literal))
    }

    fn parse_literal(&mut self) -> PResult<Option<Literal>> {
        let Some(s) = self.consume_as(Token::String) else { return Ok(None) };
        let span = self.last_span();
        let content = helper::parse_quoted_text(s).map_err(|k| k.at(span))?;
        Ok(Some(Literal::new(content.to_string(), span)))
    }

    /// Parses a char set, surrounded by `[` `]`. This was previously called a
    /// "char class", but that name is ambiguous and is being phased out.
    ///
    /// This function does _not_ parse exclamation marks in front of a char
    /// class, because negation is handled separately.
    fn parse_char_set(&mut self) -> PResult<Option<Rule>> {
        if self.consume(Token::OpenBracket) {
            let start_span = self.last_span();

            if self.consume(Token::Caret) {
                return Err(PEK::CharClass(CharClassError::CaretInGroup).at(self.last_span()));
            }

            let inner = self.parse_char_set_inner()?;

            self.expect(Token::CloseBracket).map_err(|p| {
                PEK::Expected("character class, string, code point, Unicode property or `]`")
                    .at(p.span)
            })?;
            let span = start_span.join(self.last_span());

            if inner.is_empty() {
                return Err(PEK::CharClass(CharClassError::Empty).at(span));
            }

            Ok(Some(Rule::CharClass(CharClass::new(inner, span, self.is_unicode_aware))))
        } else {
            Ok(None)
        }
    }

    /// Parses a char group, i.e. the contents of a char set. This is a sequence
    /// of characters, character classes, character ranges or Unicode
    /// properties. Some of them can be negated.
    fn parse_char_set_inner(&mut self) -> PResult<Vec<GroupItem>> {
        let mut items = Vec::new();
        loop {
            let mut nots_span = self.span();
            let mut nots = 0usize;
            while self.consume(Token::Not) {
                nots += 1;
                nots_span = nots_span.join(self.last_span());
            }

            let group = if let Some(group) = self.parse_char_group_chars_or_range()? {
                if nots > 0 {
                    return Err(PEK::UnallowedNot.at(nots_span));
                }
                group
            } else if let Some(group) = self.parse_char_group_ident(!nots.is_multiple_of(2))? {
                if nots > 1 {
                    return Err(PEK::UnallowedMultiNot(nots).at(nots_span));
                }
                group
            } else if nots > 0 {
                return Err(PEK::ExpectedToken(Token::Identifier).at(self.span()));
            } else {
                break;
            };
            items.extend(group);
        }

        Ok(items)
    }

    /// Parses an identifier or dot in a char set
    fn parse_char_group_ident(&mut self, negative: bool) -> PResult<Option<Vec<GroupItem>>> {
        if !self.consume(Token::Identifier) {
            if let Some(name) = self.consume_as(Token::ReservedName) {
                return Err(PEK::UnexpectedKeyword(name.to_owned()).at(self.last_span()));
            }
            return Ok(None);
        }
        let span = self.last_span();

        let before_colon = self.source_at(span);
        let after_colon = if self.consume(Token::Colon) {
            Some(self.expect_as(Token::Identifier)?)
        } else {
            None
        };
        let (kind, name, span) = match after_colon {
            Some(name) => (Some(before_colon), name, span.join(self.last_span())),
            None => (None, before_colon, span),
        };

        let item =
            CharGroup::try_from_group_name(kind, name, negative, span).map_err(|e| e.at(span))?;

        Ok(Some(item))
    }

    /// Parses a string literal or a character range in a char set, e.g. `"axd"`
    /// or `'0'-'7'`.
    fn parse_char_group_chars_or_range(&mut self) -> PResult<Option<Vec<GroupItem>>> {
        let span1 = self.span();
        let Some(first) = self.parse_string_or_char()? else {
            return Ok(None);
        };

        if self.consume(Token::Dash) {
            let span2 = self.span();
            let Some(last) = self.parse_string_or_char()? else {
                return Err(PEK::Expected("code point or character").at(self.span()));
            };

            if let StringOrChar::Char { is_shorthand: true, c } = first {
                self.add_warning(
                    ParseWarningKind::Deprecation(DeprecationWarning::ShorthandInRange(c))
                        .at(span1),
                );
            }
            if let StringOrChar::Char { is_shorthand: true, c } = last {
                self.add_warning(
                    ParseWarningKind::Deprecation(DeprecationWarning::ShorthandInRange(c))
                        .at(span2),
                );
            }

            let first = first.to_char().map_err(|e| e.at(span1))?;
            let last = last.to_char().map_err(|e| e.at(span2))?;

            let group = CharGroup::try_from_range(first, last).ok_or_else(|| {
                PEK::CharClass(CharClassError::NonAscendingRange(first, last)).at(span1.join(span2))
            })?;
            Ok(Some(group))
        } else {
            let group = match first {
                StringOrChar::String(s) => {
                    let chars = helper::parse_quoted_text(s).map_err(|k| k.at(span1))?;
                    chars.chars().map(GroupItem::Char).collect()
                }
                StringOrChar::Char { c, .. } => vec![GroupItem::Char(c)],
            };
            Ok(Some(group))
        }
    }

    fn parse_string_or_char(&mut self) -> PResult<Option<StringOrChar<'i>>> {
        let res = if let Some(s) = self.consume_as(Token::String) {
            StringOrChar::String(s)
        } else if let Some((c, _)) = self.parse_code_point()? {
            StringOrChar::Char { c, is_shorthand: false }
        } else if let Some(c) = self.parse_special_char() {
            StringOrChar::Char { c, is_shorthand: true }
        } else {
            return Ok(None);
        };
        Ok(Some(res))
    }

    fn parse_code_point(&mut self) -> PResult<Option<(char, Span)>> {
        let Some(cp) = self.consume_as(Token::CodePoint) else { return Ok(None) };
        let span = self.last_span();
        let trimmed_u = cp[1..].trim_start();
        if !trimmed_u.starts_with('+') {
            let warning = DeprecationWarning::Unicode(cp.into());
            self.add_warning(ParseWarningKind::Deprecation(warning).at(span))
        }

        let hex = trimmed_u.trim_start_matches(|c: char| c == '+' || c.is_whitespace());

        u32::from_str_radix(hex, 16)
            .ok()
            .and_then(|n| char::try_from(n).ok())
            .map(|c| Some((c, span)))
            .ok_or_else(|| PEK::InvalidCodePoint.at(span))
    }

    fn parse_code_point_rule(&mut self) -> PResult<Option<Rule>> {
        let Some((c, span)) = self.parse_code_point()? else { return Ok(None) };
        let inner = vec![GroupItem::Char(c)];
        Ok(Some(Rule::CharClass(CharClass::new(inner, span, self.is_unicode_aware))))
    }

    fn parse_special_char(&mut self) -> Option<char> {
        let Some((Token::Identifier, string)) = self.peek() else { return None };
        let c = match string {
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "a" => '\u{07}',
            "e" => '\u{1B}',
            "f" => '\u{0C}',
            _ => return None,
        };
        self.advance();
        Some(c)
    }

    /// Parses a boundary. For start and end, there are two syntaxes: `^` and `$`.
    /// Word boundaries are `%`.
    ///
    /// The deprecated syntax issues a warning.
    ///
    /// This function does _not_ parse negated negated word boundaries (`!%`),
    /// since negation is handled elsewhere. It also does _not_ parse the
    /// `Start` and `End` global variables.
    fn parse_boundary(&mut self) -> Option<Rule> {
        let span = self.span();
        let kind = if self.consume(Token::Caret) {
            BoundaryKind::Start
        } else if self.consume(Token::Dollar) {
            BoundaryKind::End
        } else if self.consume(Token::Percent) {
            BoundaryKind::Word
        } else if self.consume(Token::AngleLeft) {
            BoundaryKind::WordStart
        } else if self.consume(Token::AngleRight) {
            BoundaryKind::WordEnd
        } else {
            return None;
        };
        Some(Rule::Boundary(Boundary::new(kind, self.is_unicode_aware, span)))
    }

    /// Parses a reference. Supported syntaxes are `::name`, `::3`, `::+3` and
    /// `::-3`.
    fn parse_reference(&mut self) -> PResult<Option<Rule>> {
        if self.consume(Token::DoubleColon) {
            let start_span = self.last_span();

            let target = if self.consume(Token::Plus) {
                let num = self.expect_number::<i32>()?;
                ReferenceTarget::Relative(num)
            } else if self.consume(Token::Dash) {
                let num = self.expect_number::<i32>()?;
                // negating from positive to negative can't overflow, luckily
                ReferenceTarget::Relative(-num)
            } else if let Some(num) = self.consume_number(MAX_GROUP_NUMBER)? {
                ReferenceTarget::Number(num)
            } else {
                // TODO: Better diagnostic for `::let`
                let name = self
                    .expect_as(Token::Identifier)
                    .map_err(|p| PEK::Expected("number or group name").at(p.span))?;
                ReferenceTarget::Named(name.to_string())
            };

            let span = start_span.join(self.last_span());
            Ok(Some(Rule::Reference(Reference::new(target, span))))
        } else {
            Ok(None)
        }
    }

    fn parse_range(&mut self) -> PResult<Option<Rule>> {
        if self.consume_reserved("range") {
            let span_start = self.last_span();

            let first = self.expect_as(Token::String)?;
            let span_1 = self.last_span();
            self.expect(Token::Dash)?;
            let second = self.expect_as(Token::String)?;
            let span_2 = self.last_span();

            let radix = if self.consume_reserved("base") {
                let n = self.expect_number()?;
                let span = self.last_span();
                if n > 36 {
                    return Err(PEK::Number(NumberError::TooLarge).at(span));
                } else if n < 2 {
                    return Err(PEK::Number(NumberError::TooSmall).at(span));
                }
                n
            } else {
                10u8
            };

            let span = span_start.join(self.last_span());

            let start = helper::parse_number(helper::strip_first_last(first), radix)
                .map_err(|k| PEK::from(k).at(span_1))?;
            let end = helper::parse_number(helper::strip_first_last(second), radix)
                .map_err(|k| PEK::from(k).at(span_2))?;

            if start.is_empty() || end.is_empty() {
                let span = if start.is_empty() { span_1 } else { span_2 };
                return Err(PEK::Number(NumberError::Empty).at(span));
            }

            if start.len() > end.len() || (start.len() == end.len() && start > end) {
                return Err(PEK::RangeIsNotIncreasing.at(span_1.join(span_2)));
            }

            if start.len() != end.len()
                && (helper::has_leading_zero(&start) || helper::has_leading_zero(&end))
            {
                return Err(PEK::RangeLeadingZeroesVariableLength.at(span_1.join(span_2)));
            }

            Ok(Some(Rule::Range(Range::new(
                start.into_boxed_slice(),
                end.into_boxed_slice(),
                radix,
                span,
            ))))
        } else {
            Ok(None)
        }
    }

    /// Parses an unescaped regex expression (`regex "[test]"`)
    fn parse_regex(&mut self) -> PResult<Option<Rule>> {
        if self.consume_reserved("regex") {
            let span_start = self.last_span();
            let lit = self.expect_as(Token::String)?;
            let span_end = self.last_span();

            let content = helper::parse_quoted_text(lit).map_err(|k| k.at(span_end))?;

            let span = span_start.join(span_end);
            Ok(Some(Rule::Regex(Regex::new(content.to_string(), span))))
        } else {
            Ok(None)
        }
    }

    /// Parses a variable (usage site).
    fn parse_variable(&mut self) -> PResult<Option<Rule>> {
        let Some(ident) = self.consume_as(Token::Identifier) else { return Ok(None) };
        let span1 = self.last_span();
        let rule = Rule::Variable(Variable::new(ident, span1));
        if let Some((Token::Equals, span2)) = self.peek_pair() {
            return Err(PEK::MissingLetKeyword.at(span1.join(span2)));
        }
        Ok(Some(rule))
    }

    /// Parses the dot
    fn parse_dot(&mut self) -> Option<Rule> {
        if self.consume(Token::Dot) { Some(Rule::Dot) } else { None }
    }

    /// Parses the `recursion` keyword
    fn parse_recursion(&mut self) -> Option<Rule> {
        if self.consume_reserved("recursion") {
            Some(Rule::Recursion(Recursion { span: self.last_span() }))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
enum StringOrChar<'i> {
    String(&'i str),
    Char { c: char, is_shorthand: bool },
}

impl StringOrChar<'_> {
    fn to_char(self) -> Result<char, PEK> {
        Err(PEK::CharString(match self {
            StringOrChar::Char { c, .. } => return Ok(c),
            StringOrChar::String(s) => {
                let s = helper::parse_quoted_text(s)?;
                let mut iter = s.chars();
                match iter.next() {
                    Some(c) if iter.next().is_none() => return Ok(c),
                    Some(_) => CharStringError::TooManyCodePoints,
                    _ => CharStringError::Empty,
                }
            }
        }))
    }
}

trait TryOptionExt<T> {
    fn try_or_else<E>(self, f: impl FnMut() -> Result<Option<T>, E>) -> Result<Option<T>, E>;
}

impl<T> TryOptionExt<T> for Option<T> {
    #[inline(always)]
    fn try_or_else<E>(self, mut f: impl FnMut() -> Result<Option<T>, E>) -> Result<Option<T>, E> {
        match self {
            Some(val) => Ok(Some(val)),
            None => f(),
        }
    }
}
