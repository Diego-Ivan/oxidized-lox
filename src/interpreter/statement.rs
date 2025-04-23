use crate::expression::Expression;
use crate::token::Token;

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    VariableDeclaration {
        name: String,
        initializer: Option<Expression>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Token>,
        body: Box<Statement>,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
}
