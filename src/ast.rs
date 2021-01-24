use anyhow::Context;

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
    Conditional(Conditional),
    Assignment(Assignment),
    // Derived
    MacroUse(MacroUse),
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

enum Definition {
    DefVar(Variable, Box<Expression>),
    DefLambda(Variable, Vec<Variable>, LambdaBody),
    Multiple(Vec<Definition>),
}

struct Conditional {
    test: Box<Expression>,
    consequent: Box<Expression>,
    alternate: Option<Box<Expression>>,
}

struct Assignment {
    variable: Variable,
    expression: Box<Expression>,
}

struct MacroUse {
    keyword: String,
    datum: Vec<Datum>,
}

enum MacroBlock {
    Let(Syntax),
    LetRec(Syntax),
}

struct Syntax {
    syntax_spec: Vec<SyntaxSpec>,
    body: LambdaBody,
}

struct SyntaxSpec {
    keyword: String,
    transformer_spec: TransformerSpec,
}

struct TransformerSpec {
    identifiers: Vec<String>,
    syntax_rules: Vec<SyntaxRule>,
}

struct SyntaxRule {
    pattern: Pattern,
    template: Template,
}

struct Pattern {}

struct Template {}
