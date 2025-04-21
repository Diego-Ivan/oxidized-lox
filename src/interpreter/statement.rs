use crate::expression::Expression;

pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Declaration {
        name: String,
        initializer: Option<Expression>,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
}
