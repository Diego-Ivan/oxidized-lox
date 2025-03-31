use crate::expression::Expression;

pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Declaration {
        name: String,
        initializer: Option<Expression>,
    },
}
