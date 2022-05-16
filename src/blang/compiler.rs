use super::{lexer::Lexer, lexer::TokenType};

pub struct Compiler {}

impl Compiler {
    /*fn compile(source: &str) -> Result<Chunk, CompilerError> {
        /et mut chunk = Chunk::new();
        let mut parser = Parser::new(source);
        while let Some(statement) = parser.parse_statement() {
            //println!("{:?}", statement);
            chunk.write_statement(statement);
        }
        Ok(chunk)
    }
    */
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: String) {
        let mut lexer = Lexer::new(source);
        let mut line = 0;
        loop {
            match lexer.scan_token() {
                Ok(token) => {
                    // Print one line at a time
                    if token.line != line {
                        line = token.line;
                        print!("{} ", line);
                        line = token.line;
                    } else {
                        print!("   | ");
                    }

                    println!("{:?} {}", token.t_type, lexer.get_lexeme(&token));

                    if token.t_type == TokenType::Eof {
                        break;
                    }
                }
                Err(err) => {
                    println!("{}, {}", err.line, err.message);
                    break;
                }
            }
        }
    }
}
