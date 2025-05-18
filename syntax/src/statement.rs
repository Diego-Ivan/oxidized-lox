use crate::expression::Expression;
use crate::token::Token;

pub type Block = Vec<Statement>;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    VariableDeclaration {
        name: String,
        initializer: Option<Expression>,
    },
    FunctionDeclaration(Function),
    Block(Block),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        initializer: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Statement>,
    },
    ClassDeclaration {
        name: String,
        methods: Vec<Function>,
    },
    Return {
        keyword: Token,
        expression: Option<Expression>,
    },
    Break {
        keyword: Token,
    },
    Continue {
        keyword: Token,
    },
}
