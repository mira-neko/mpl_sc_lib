use std::fmt;
mod ast;
mod ast_indexed;

pub struct Parser(ast_indexed::AstIndexed);

impl From<&str> for Parser {
    fn from(s: &str) -> Parser {
        Parser(ast_indexed::AstIndexed::from(ast::Ast::from(s)))
    }
}

impl fmt::Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Parser {
    #[allow(dead_code)]
    pub fn eval<F: FnMut() -> Option<f64>>(&self, input: &mut F, debug: bool) -> Option<()> {
        for res in mplvm_parser::parse(&self.0.to_string(), input, debug) {
            let Ok(opt) = res else {
                return None
            };

            if let Some(val) = opt {
                println!("{val}");
            }
        }

        Some(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn print_2_plus_2() {
        use super::Parser;

        let source = "print(2 + 2)\n";

        let program = Parser::from(source);

        assert!(program.to_string() == "psh 2\npsh 2\nadd\npek\npop\n");
    }
}
