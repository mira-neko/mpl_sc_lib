use std::fmt;
mod ast;
mod ast_indexed;
mod ir;

pub struct Parser(ir::Ir);

impl From<&str> for Parser {
    fn from(s: &str) -> Parser {
        Parser(ir::Ir::from(ast_indexed::AstIndexed::from(ast::Ast::from(s))))
    }
}

impl fmt::Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        mpl_vm::Program::from((self.0.codegen(), || None)).fmt(f)
    }
}

impl Parser {
    #[allow(dead_code)]
    pub fn eval<F: FnMut() -> Option<f64>>(self, input: &mut F, debug: bool) -> Option<()> {
        for res in mpl_vm::Program::from((self.0.codegen(), input, debug)) {
            if let Some(val) = res.ok()? {
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
