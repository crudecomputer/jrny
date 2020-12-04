use std::mem;
use unicode_segmentation::UnicodeSegmentation;

use crate::statements::Statement;

mod states;
use states::*;

/// A simple pseudo-pushdown automaton that generates a vec of individual
/// statements by accepting one grapheme at a time.
pub struct Parser {
    pub statements: Vec<Statement>,

    carried: String,
    state: Box<dyn State>,
}

impl Parser {
    pub fn parse(input: &str) -> Vec<Statement> {
        let mut parser = Self::new();

        for grapheme in input.graphemes(true) {
            parser.accept(grapheme);
        }

        // If the parser handled white-space better, the extra allocations
        // here would not be necessary... TODO
        parser.statements.iter()
            .map(|stmt| Statement(stmt.0.trim().to_string()))
            .filter(|stmt| !stmt.0.is_empty())
            .collect()
    }

    fn new() -> Self {
        Self {
            carried: String::new(),
            statements: vec![],
            state: Box::new(Start),
        }
    }

    fn accept(&mut self, next: &str) {
        if self.statements.len() == 0 {
            self.statements.push(Statement::default());
        }

        // It's safe to unwrap here, as we're guaranteed to have at least 1
        let current_statement = self.statements.last_mut().expect("the unexpected");

        if next == ";" && self.state.can_terminate() {
            current_statement.0.push_str(&self.carried);
            self.carried = String::new();

            self.statements.push(Statement::default());
            self.state = Box::new(Start);

            return;
        }

        // This is KIND OF like popping a value off a pushdown stack.
        // Maybe it should more explicitly pass it separately from the incoming grapheme.
        let next = {
            let mut carried = mem::replace(&mut self.carried, String::new());
            carried.push_str(&next);
            carried
        };

        let (action, next_state) = (&self.state).accept(&next);

        self.state = next_state;

        match action {
            Action::Append => { current_statement.0.push_str(&next); },
            Action::Carry  => { self.carried = next; },
            Action::Ignore => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_statement() {
        let empty = vec![];

        assert_eq!(Parser::parse(""), empty);
        assert_eq!(Parser::parse("  "), empty);
        assert_eq!(Parser::parse("  \n  \n  "), empty);
        assert_eq!(Parser::parse(" ;; ; ;  ;"), empty);
    }

    #[test]
    fn single_statement() {
        assert_eq!(
            Parser::parse("anything really, does not matter"),
            vec![
                Statement("anything really, does not matter".to_string()),
            ],
        );
    }

    #[test]
    fn single_with_embedded_semicolons() {
        assert_eq!(
            Parser::parse("one thing ';' and two things \";\""),
            vec![
                Statement("one thing ';' and two things \";\"".to_string()),
            ],
        );
    }

    #[test]
    fn single_with_semicolon_after_carry() {
        assert_eq!(
            Parser::parse("one thing -; two thing"),
            vec![
                Statement("one thing -".to_string()),
                Statement("two thing".to_string()),
            ],
        );
    }

    #[test]
    fn multiple_without_embedded() {
        assert_eq!(
            Parser::parse("  one thing  ; two things "),
            vec![
                Statement("one thing".to_string()),
                Statement("two things".to_string()),
            ],
        );
    }

    #[test]
    fn multiple_with_embedded() {
        assert_eq!(
            Parser::parse(r#"  one ';' ";" thing  ; two things "#),
            vec![
                Statement(r#"one ';' ";" thing"#.to_string()),
                Statement("two things".to_string()),
            ],
        );
    }

    #[test]
    fn quoted_with_semicolons() {
        assert_eq!(
            Parser::parse(r#" '";'"  "#),
            vec![
                Statement(r#"'";'""#.to_string()),
            ],
        );
        assert_eq!(
            Parser::parse(r#" '"';"  "#),
            vec![
                Statement(r#"'"'"#.to_string()),
                Statement(r#"""#.to_string()),
            ],
        );
        assert_eq!(
            Parser::parse(r#" a ';' b ";" c '";"' d "';'" e    "#),
            vec![
                Statement(r#"a ';' b ";" c '";"' d "';'" e"#.to_string()),
            ],
        );
    }

    #[test]
    fn inline_comments_with_semicolon_on_own_line() {
        assert_eq!(
            Parser::parse("
this is one statement;
this is
-- ;
another statement
            "),
            vec![
                Statement("this is one statement".to_string()),
                Statement("this is\n\nanother statement".to_string()),
            ],
        );
    }

    #[test]
    fn inline_comments_with_semicolon_trailing() {
        assert_eq!(
            Parser::parse("
this is one statement;
this is -- ;
another statement
            "),
            vec![
                Statement("this is one statement".to_string()),
                Statement("this is \nanother statement".to_string()),
            ],
        );
    }

    #[test]
    fn block_comments_with_semicolons_own_line() {
        assert_eq!(
            Parser::parse("
this is one statement;
this is
/* ; */
another statement
            "),
            vec![
                Statement("this is one statement".to_string()),
                Statement("this is\n\nanother statement".to_string()),
            ],
        );
    }

    #[test]
    fn block_comments_with_semicolons_trailing() {
        assert_eq!(
            Parser::parse("
this is one statement;
this is /* ; */
another statement
            "),
            vec![
                Statement("this is one statement".to_string()),
                Statement("this is \nanother statement".to_string()),
            ],
        );
    }

    #[test]
    fn block_comments_with_semicolons_inline() {
        assert_eq!(
            Parser::parse("
this is one statement;
this is /* ; */ another statement
            "),
            vec![
                Statement("this is one statement".to_string()),
                Statement("this is  another statement".to_string()),
            ],
        );
    }
}
