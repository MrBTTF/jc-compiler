use crate::emitter::ast::*;

pub struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_statement_list(&mut self, statement_list: &StatementList) -> String {
        statement_list
            .0
            .iter()
            .map(|stmt| self.visit_statement(stmt))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn visit_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Expression(expr) => self.visit_expression(expr),
            Statement::Assignment(ident, expr) => {
                let s1 = self.visit_ident(ident);
                let s2 = self.visit_expression(expr);
                s1 + " = " + &s2
            }
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> String {
        match expression {
            Expression::Literal(literal) => self.visit_literal(literal),
            Expression::Call(ident, expr) => {
                let s1 = self.visit_ident(ident);
                let s2 = self.visit_expression(expr);
                s1 + "(" + &s2 + ")"
            }
        }
    }

    fn visit_literal(&mut self, literal: &Literal) -> String {
        match literal {
            Literal::Ident(ident) => self.visit_ident(ident),
            Literal::String(str) => format!("\"{str}\""),
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