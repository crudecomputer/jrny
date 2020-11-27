// read a revision
// parse into list of commands
// error if encountering begin; rollback; commit; savepoint;

#[derive(Debug, Default, PartialEq)]
pub struct Statement(pub String);

#[derive(Debug)]
pub struct StatementGroup {
    pub statements: Vec<Statement>,
}

impl StatementGroup {
    pub fn new(text: &str) -> Result<Self, String> {
        Ok(Self {
            statements: parse(text)?,
        })
    }
}

pub fn parse(text: &str) -> Result<Vec<Statement>, String> {
    let mut parser = Parser::new();

    let without_comments: String = text.lines()
        .filter(|l| !l.trim().starts_with("--"))
        .fold(String::new(), |a, b| a + b + "\n");

    // TODO how bad is using `chars` for non-UTF char sets, of which there
    // are a ton supported by postgres?
    // See: https://www.postgresql.org/docs/13/multibyte.html#MULTIBYTE-CHARSET-SUPPORTED
    for c in without_comments.chars() {
        parser.accept(c);
    }

    // If the parser handled white-space better, the extra allocations
    // here would not be necessary... TODO
    let statements: Vec<Statement> = parser.statements.iter()
        .map(|stmt| Statement(stmt.0.trim().to_string()))
        .filter(|stmt| !stmt.0.is_empty())
        .collect();

    // Transaction-management commands should cause immediate errors,
    // and thankfully it's just exact keyword matching at the start
    // (provided the string is TRIMMED) and it doesn't matter if
    // they're embedded inside a string or delimited identifier at all.
    for s in &statements {
        let lowered = s.0.chars()
            .take(10)
            .collect::<String>()
            .to_lowercase();

        for command in ["begin", "savepoint", "rollback", "commit"].iter() {
            if lowered.starts_with(command) {
                return Err(format!(
                    "{} command is not supported in a revision",
                    command.to_uppercase(),
                ));
            }
        }
    }

    Ok(statements)
}

struct Parser {
    statements: Vec<Statement>,
    in_string: bool,
    in_delimited_identifier: bool,
}

impl Parser {
    fn new() -> Self {
        Self {
            statements: vec![],
            in_string: false,
            in_delimited_identifier: false,
        }
    }

    fn accept(&mut self, c: char) {
        // A single quote can open or close a text string, but ONLY if
        // it's not embedded in a delimited identifier
        if c == '\'' && !self.in_delimited_identifier {
            self.in_string = !self.in_string;
        }

        // Likewise, a double quote can open or close a delimited identifer,
        // but only if it's not inside a text string
        if c == '"' && !self.in_string {
            self.in_delimited_identifier = !self.in_delimited_identifier;
        }

        // Meanwhile, back at the ranch, a semicolon ends a statement
        // only if it's outside of text strings or quoted identifiers.
        // A semicolon ending a statement is the only time the character
        // SHOULD NOT be appended; instead, it should "end" the current
        // statement by creating a new one.
        if c == ';' && !self.in_string && !self.in_delimited_identifier {
            self.statements.push(Statement::default());

            return;
        }

        if self.statements.len() == 0 {
            self.statements.push(Statement::default());
        }

        self.statements
            .last_mut()
            .unwrap()
            .0.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let empty: Vec<Statement> = vec![];

        assert_eq!(parse("").unwrap(), empty);
        assert_eq!(parse("  ").unwrap(), empty);
        assert_eq!(parse("  \n  \n  ").unwrap(), empty);
        assert_eq!(parse(" ;; ; ;  ;").unwrap(), empty);
    }

    #[test]
    fn test_single() {
        assert_eq!(
            parse("anything really, does not matter").unwrap(),
            vec![
                Statement("anything really, does not matter".to_string()),
            ],
        );
    }

    #[test]
    fn test_single_with_embedded_semicolons() {
        assert_eq!(
            parse("one thing ';' and two things \";\"").unwrap(),
            vec![
                Statement("one thing ';' and two things \";\"".to_string()),
            ],
        );
    }

    #[test]
    fn test_multiple_without_embedded() {
        assert_eq!(
            parse("  one thing  ; two things ").unwrap(),
            vec![
                Statement("one thing".to_string()),
                Statement("two things".to_string()),
            ],
        );
    }

    #[test]
    fn test_quoted_with_semicolons() {
        assert_eq!(
            parse(r#" '";'"  "#).unwrap(),
            vec![
                Statement(r#"'";'""#.to_string()),
            ]
        );
        assert_eq!(
            parse(r#" '"';"  "#).unwrap(),
            vec![
                Statement(r#"'"'"#.to_string()),
                Statement(r#"""#.to_string()),
            ]
        );
        assert_eq!(
            parse(r#" a ';' b ";" c '";"' d "';'" e    "#).unwrap(),
            vec![
                Statement(r#"a ';' b ";" c '";"' d "';'" e"#.to_string()),
            ]
        );
    }

    #[test]
    fn test_errors_from_transaction_commands() {
        let err = |cmd| Err(format!(
            "{} command is not supported in a revision",
            cmd,
        ));

        assert_eq!(parse(" beGIN "),         err("BEGIN"));
        assert_eq!(parse("one; begin; two"), err("BEGIN"));
        assert_eq!(parse("ONE; BEGIN; TWO"), err("BEGIN"));

        assert_eq!(parse("  savEPOint "),        err("SAVEPOINT"));
        assert_eq!(parse("one; savepoint; two"), err("SAVEPOINT"));
        assert_eq!(parse("ONE; SAVEPOINT; TWO"), err("SAVEPOINT"));

        assert_eq!(parse("  rOLLBack "),        err("ROLLBACK"));
        assert_eq!(parse("one; rollback; two"), err("ROLLBACK"));
        assert_eq!(parse("ONE; ROLLBACK; TWO"), err("ROLLBACK"));

        assert_eq!(parse("  coMMIt "),        err("COMMIT"));
        assert_eq!(parse("one; commit; two"), err("COMMIT"));
        assert_eq!(parse("ONE; COMMIT; TWO"), err("COMMIT"));

        assert_eq!(parse("begin; rollback; savepoint; commit"), err("BEGIN"));
        assert_eq!(parse("rollback; begin; savepoint; commit"), err("ROLLBACK"));
        assert_eq!(parse("savepoint; begin; rollback; commit"), err("SAVEPOINT"));
        assert_eq!(parse("commit; begin; rollback; commit"),    err("COMMIT"));
    }
}
