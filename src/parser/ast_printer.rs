use crate::emitter::ast::*;

use super::ident;

pub struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_statement_list(&mut self, statement_list: &StatementList) -> String {
        statement_list
            .stmts
            .iter()
            .map(|stmt| self.visit_statement(stmt))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn visit_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Expression(expr) => self.visit_expression(expr),
            Statement::Declaration(Declaration(ident, expr, assign_type)) => {
                let s1 = self.visit_ident(ident);
                let s2 = self.visit_expression(expr);
                format!("{assign_type} {s1} = {s2}")
            }
            Statement::Assignment(Assignment(ident, expr)) => {
                let s1 = self.visit_ident(ident);
                let s2 = self.visit_expression(expr);
                format!("{s1} = {s2}")
            }
            Statement::FuncDefinition(FuncDefinition(name, args, return_type, stmts)) => {
                let s_name = self.visit_ident(name);
                let s_args = args.iter().fold(String::new(), |mut acc, a| {
                    let arg_name = self.visit_ident(&a.name);
                    acc.push_str(arg_name.as_str());
                    acc.push_str(" ");
                    acc.push_str(&a._type.to_string());
                    acc
                });

                let mut result = format!("{s_name} ({s_args})");
                if let Some(return_type) = return_type {
                    let s_return_type = self.visit_ident(return_type);
                    result = format!("{result} {s_return_type}")
                }
                let s_stmts = self.visit_statement_list(stmts).replace("\n", "\n\t");

                format!("func {result}{{\n\t{s_stmts} \n}}")
            }
            Statement::Scope(stmts) => {
                format!("{{\n{}\n}}", self.visit_statement_list(stmts))
            }
            Statement::ControlFlow(_) => format!(""),
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> String {
        match expression {
            Expression::Ident(ident) => self.visit_ident(ident),
            Expression::Literal(literal) => self.visit_literal(literal),
            Expression::Call(ident, exprs) => {
                let s1 = self.visit_ident(ident);
                let s2 = exprs
                    .iter()
                    .map(|expr| self.visit_expression(expr))
                    .collect::<Vec<String>>()
                    .join(", ");
                s1 + "(" + &s2 + ")"
            }
            Expression::Loop(l) => {
                let s = format!("for {} in {}..{}", l.var.value, l.start, l.end);
                let body = self.visit_statement_list(&l.body);

                s + " {\n" + &body + "\n}\n"
            }
        }
    }

    fn visit_literal(&mut self, literal: &Literal) -> String {
        match literal {
            Literal::String(str) => format!(".{str}").replace("\n", "\\n"),
            Literal::Number(number) => self.visit_number(number),
        }
    }

    fn visit_ident(&mut self, ident: &Ident) -> String {
        ident.value.to_owned()
    }

    fn visit_number(&mut self, number: &Number) -> String {
        number.value.to_string()
    }
}
