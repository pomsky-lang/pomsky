use pomsky_syntax::exprs::{self, Rule};

pub(crate) enum NestingKind {
    Group,
    Alternation,
    Intersection,
    Repetition,
    Lookaround,
    StmtExpr,
    Let,
    Negation,
}

fn visit<V: RuleVisitor<E>, E>(rule: &Rule, visitor: &mut V) -> Result<(), E> {
    match rule {
        Rule::Literal(l) => visitor.visit_literal(l),
        Rule::CharClass(c) => visitor.visit_char_class(c),
        Rule::Group(g) => {
            visitor.visit_group(g)?;
            visitor.down(NestingKind::Group);
            for rule in &g.parts {
                visit(rule, visitor)?;
            }
            visitor.up(NestingKind::Group);
            Ok(())
        }
        Rule::Alternation(a) => {
            visitor.visit_alternation(a)?;
            visitor.down(NestingKind::Alternation);
            for rule in &a.rules {
                visit(rule, visitor)?;
            }
            visitor.up(NestingKind::Alternation);
            Ok(())
        }
        Rule::Intersection(i) => {
            visitor.visit_intersection(i)?;
            visitor.down(NestingKind::Intersection);
            for rule in &i.rules {
                visit(rule, visitor)?;
            }
            visitor.up(NestingKind::Intersection);
            Ok(())
        }
        Rule::Repetition(r) => {
            visitor.visit_repetition(r)?;
            visitor.down(NestingKind::Repetition);
            visit(&r.rule, visitor)?;
            visitor.up(NestingKind::Repetition);
            Ok(())
        }
        Rule::Boundary(b) => visitor.visit_boundary(b),
        Rule::Lookaround(l) => {
            visitor.visit_lookaround(l)?;
            visitor.down(NestingKind::Lookaround);
            visit(&l.rule, visitor)?;
            visitor.up(NestingKind::Lookaround);
            Ok(())
        }
        Rule::Variable(v) => visitor.visit_variable(v),
        Rule::Reference(r) => visitor.visit_reference(r),
        Rule::Range(r) => visitor.visit_range(r),
        Rule::StmtExpr(s) => {
            visitor.visit_statement(&s.stmt)?;
            if let exprs::Stmt::Let(l) = &s.stmt {
                visitor.down(NestingKind::Let);
                visit(&l.rule, visitor)?;
                visitor.up(NestingKind::Let);
            }
            visitor.down(NestingKind::StmtExpr);
            visit(&s.rule, visitor)?;
            visitor.up(NestingKind::StmtExpr);
            Ok(())
        }
        Rule::Negation(n) => {
            visitor.visit_negation(n)?;
            visitor.down(NestingKind::Negation);
            visit(&n.rule, visitor)?;
            visitor.up(NestingKind::Negation);
            Ok(())
        }
        Rule::Regex(r) => visitor.visit_regex(r),
        Rule::Recursion(r) => visitor.visit_recursion(r),
        Rule::Grapheme => visitor.visit_grapheme(),
        Rule::Codepoint => visitor.visit_codepoint(),
        Rule::Dot => visitor.visit_dot(),
    }
}

#[allow(unused_variables)]
pub(crate) trait RuleVisitor<E> {
    fn visit_rule(&mut self, rule: &exprs::Rule) -> Result<(), E>
    where
        Self: Sized,
    {
        visit(rule, self)
    }

    fn down(&mut self, kind: NestingKind) {}

    fn up(&mut self, kind: NestingKind) {}

    fn visit_literal(&mut self, literal: &exprs::Literal) -> Result<(), E> {
        Ok(())
    }

    fn visit_char_class(&mut self, char_class: &exprs::CharClass) -> Result<(), E> {
        Ok(())
    }

    fn visit_group(&mut self, group: &exprs::Group) -> Result<(), E> {
        Ok(())
    }

    fn visit_alternation(&mut self, alt: &exprs::Alternation) -> Result<(), E> {
        Ok(())
    }

    fn visit_intersection(&mut self, int: &exprs::Intersection) -> Result<(), E> {
        Ok(())
    }

    fn visit_repetition(&mut self, repetition: &exprs::Repetition) -> Result<(), E> {
        Ok(())
    }

    fn visit_boundary(&mut self, boundary: &exprs::Boundary) -> Result<(), E> {
        Ok(())
    }

    fn visit_lookaround(&mut self, lookaround: &exprs::Lookaround) -> Result<(), E> {
        Ok(())
    }

    fn visit_variable(&mut self, variable: &exprs::Variable) -> Result<(), E> {
        Ok(())
    }

    fn visit_reference(&mut self, reference: &exprs::Reference) -> Result<(), E> {
        Ok(())
    }

    fn visit_range(&mut self, range: &exprs::Range) -> Result<(), E> {
        Ok(())
    }

    fn visit_statement(&mut self, statement: &exprs::Stmt) -> Result<(), E> {
        Ok(())
    }

    fn visit_negation(&mut self, negation: &exprs::Negation) -> Result<(), E> {
        Ok(())
    }

    fn visit_regex(&mut self, regex: &exprs::Regex) -> Result<(), E> {
        Ok(())
    }

    fn visit_recursion(&mut self, recursion: &exprs::Recursion) -> Result<(), E> {
        Ok(())
    }

    fn visit_grapheme(&mut self) -> Result<(), E> {
        Ok(())
    }

    fn visit_codepoint(&mut self) -> Result<(), E> {
        Ok(())
    }

    fn visit_dot(&mut self) -> Result<(), E> {
        Ok(())
    }
}
