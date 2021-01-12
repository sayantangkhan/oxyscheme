use crate::{lexer::LispNum, parser::Datum};

enum Expression {
    Variable(Variable),
    Boolean(bool),
    Number(LispNum),
    Character(char),
    String(String),
    Quotation(Box<Expression>),
    ProcedureCall(ProcedureCall),
    Lambda(Lambda),
    // Conditional
    // Assignment
    // Derived
    // MacroUse
    // MacroBlock
}

struct Variable {
    name: String,
}

struct ProcedureCall {
    operator: Box<Expression>,
    operand: Vec<Expression>,
}

struct Lambda {
    arguments: LambdaArgs,
    body: LambdaBody,
}

enum LambdaArgs {
    List(Vec<Variable>),
    Atom(Variable),
    Pair(Vec<Variable>, Variable),
}

struct LambdaBody {
    definitions: Vec<Definition>,
    commands: Vec<Expression>,
    return_expression: Box<Expression>,
}

struct Definition {
    // Insert definition later
}
