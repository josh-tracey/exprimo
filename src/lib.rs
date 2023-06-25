use rslint_parser::{
    ast::{BinExpr, BinOp, CondExpr, DotExpr, Expr, Name, NameRef, UnaryExpr, UnaryOp},
    parse_text, AstNode, SyntaxKind, SyntaxNode, SyntaxNodeExt,
};

#[cfg(feature = "logging")]
use scribe_rust::Logger;

#[cfg(feature = "logging")]
use std::sync::Arc;

use std::{collections::HashMap, error::Error};

pub struct Evaluator {
    context: HashMap<String, serde_json::Value>,
    #[cfg(feature = "logging")]
    logger: Arc<Logger>,
}

impl Evaluator {
    pub fn new(
        context: HashMap<String, serde_json::Value>,
        #[cfg(feature = "logging")] logger: Arc<Logger>,
    ) -> Self {
        Evaluator {
            context,
            #[cfg(feature = "logging")]
            logger,
        }
    }

    pub fn evaluate(&self, expression: &str) -> Result<bool, Box<dyn Error>> {
        let ast = parse_text(expression, 0).syntax();
        let untyped_expr_node = ast.first_child().unwrap();

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Expression AST:\n\n{:#?}\n-----------------",
            untyped_expr_node
        ));

        let result = self.evaluate_node(&untyped_expr_node)?;

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Result: {}", result));

        Ok(result)
    }

    fn evaluate_node(&self, node: &SyntaxNode) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluting NodeKind: {:#?}, {:?}",
            node.kind(),
            node.to_string()
        ));

        let res = match node.kind() {
            SyntaxKind::EXPR_STMT => {
                let expr = node.first_child().unwrap();
                self.evaluate_node(&expr)
            }
            SyntaxKind::DOT_EXPR => self.evaluate_dot_expr(&node.try_to::<DotExpr>().unwrap()),
            SyntaxKind::NAME_REF => self.evaluate_name_ref(&node.try_to::<NameRef>().unwrap()),
            SyntaxKind::NAME => self.evaluate_name(&node.try_to::<Name>().unwrap()),
            SyntaxKind::BIN_EXPR => self.evaluate_bin_expr(&node.try_to::<BinExpr>().unwrap()),
            SyntaxKind::LITERAL => self.evaluate_literal(&node.try_to::<Expr>().unwrap()),
            SyntaxKind::COND_EXPR => self.evaluate_cond_expr(&node.try_to::<CondExpr>().unwrap()),
            SyntaxKind::IDENT => self.evaluate_identifier(&node.try_to::<Expr>().unwrap()),
            SyntaxKind::UNARY_EXPR => {
                self.evaluate_prefix_expr(&node.try_to::<UnaryExpr>().unwrap())
            }
            _ => Ok(false), // Handle other types of expressions accordingly
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "NodeKind: {:?} => {:#?}",
            node.kind(),
            res.as_ref().unwrap()
        ));

        res
    }

    fn evaluate_bin_expr(&self, bin_expr: &BinExpr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Binary Expression: {:#?}",
            bin_expr.to_string()
        ));

        let left = bin_expr.lhs();
        let right = bin_expr.rhs();

        let left_value = self.evaluate_node(left.unwrap().syntax())?;
        let right_value = self.evaluate_node(right.unwrap().syntax())?;

        let op = bin_expr.op_details();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp left_value {}", left_value));

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp right_value {}", right_value));

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("BinaryOp op_details {:?}", op));

        let result = match op {
            Some((_, BinOp::LogicalAnd)) => left_value && right_value,
            Some((_, BinOp::LogicalOr)) => left_value || right_value,
            Some((_, BinOp::Equality)) => left_value == right_value,
            Some((_, BinOp::Inequality)) => left_value != right_value,
            Some((_, BinOp::StrictEquality)) => left_value == right_value,
            Some((_, BinOp::StrictInequality)) => left_value != right_value,
            Some((_, BinOp::GreaterThan)) => left_value > right_value,
            Some((_, BinOp::LessThan)) => left_value < right_value,
            Some((_, BinOp::GreaterThanOrEqual)) => left_value >= right_value,
            Some((_, BinOp::LessThanOrEqual)) => left_value <= right_value,
            _ => false,
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Binary Result: {}", result));

        Ok(result)
    }

    fn evaluate_prefix_expr(&self, prefix_expr: &UnaryExpr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Prefix Expression: {:#?}",
            prefix_expr.to_string()
        ));
        let expr = prefix_expr.expr().unwrap();
        let expr_value = self.evaluate_node(expr.syntax())?;

        let op = prefix_expr.op_details();

        let result = match op {
            Some((_, UnaryOp::LogicalNot)) => {
                #[cfg(feature = "logging")]
                self.logger
                    .trace(&format!("UnaryOp expr_value {}", expr_value));
                let bool_value = expr_value;
                !bool_value
            }
            _ => false,
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Prefix Result: {}", result));

        Ok(result)
    }

    fn evaluate_cond_expr(&self, cond_expr: &CondExpr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Conditional Expression: {:#?}",
            cond_expr.to_string()
        ));
        let cond = cond_expr.test().unwrap();
        let true_expr = cond_expr.cons().unwrap();
        let false_expr = cond_expr.alt().unwrap();

        let cond_value = self.evaluate_node(cond.syntax())?;

        let result = match cond_value {
            true => self.evaluate_node(true_expr.syntax())?,
            false => self.evaluate_node(false_expr.syntax())?,
        };

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Conditional Result: {}", result));

        Ok(result)
    }

    fn evaluate_dot_expr(&self, dot_expr: &DotExpr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Dot Expression: {:#?}",
            dot_expr.to_string()
        ));
        let mut left = dot_expr.clone().syntax().clone();
        let mut right = dot_expr.prop().unwrap().ident_token().unwrap().to_string();

        let mut left_value = self.evaluate_node(&left)?;

        while let SyntaxKind::DOT_EXPR = left.clone().kind() {
            if let Some(parent) = left.clone().parent() {
                left = parent.clone();
                if let Some(dot_expr) = parent.try_to::<DotExpr>() {
                    let property = dot_expr.prop().unwrap().ident_token().unwrap();
                    right = format!("{}.{}", property, right);
                    left_value = self.evaluate_node(&left)?;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        match left_value {
            true => {
                let property_value = self.context.get(&right);

                match property_value {
                    Some(value) => Ok(value == "true"),
                    None => Err(Box::from("Property not found in context.")),
                }
            }
            false => Ok(false),
        }
    }

    fn evaluate_by_name(&self, identifier_name: String) -> Result<bool, Box<dyn Error>> {
        let identifier_value = self.context.get(&identifier_name);

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Identifier Value: {:#?}", identifier_value));

        match identifier_value {
            Some(serde_json::Value::Bool(b)) => Ok(*b),
            Some(_) => Err(Box::from("Identifier value is not a boolean.")),
            None => Err(Box::from("Identifier not found in context.")),
        }
    }

    fn evaluate_name(&self, name: &Name) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Name: {:#?}", name.to_string()));
        let identifier_name = name.ident_token().unwrap().to_string();
        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_name_ref(&self, name_ref: &NameRef) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Name Reference: {:#?}",
            name_ref.to_string()
        ));
        let identifier_name = name_ref.ident_token().unwrap().to_string();
        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_identifier(&self, identifier: &Expr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Identifier: {:#?}",
            identifier.to_string()
        ));
        let identifier_name = identifier.to_string();
        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_literal(&self, literal: &Expr) -> Result<bool, Box<dyn Error>> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Literal: {:#?}", literal.to_string()));

        let literal_value = literal.to_string();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Literal value: {:#?}", literal_value));

        let value: serde_json::Value =
            serde_json::from_str(&literal_value).map_err(|e| Box::new(e) as Box<dyn Error>)?;

        match value {
            serde_json::Value::Bool(b) => Ok(b),
            serde_json::Value::Number(n) => Ok(n.as_i64().unwrap() != 0),
            serde_json::Value::String(s) => Ok(!s.is_empty()),
            serde_json::Value::Array(a) => Ok(!a.is_empty()),
            serde_json::Value::Object(o) => Ok(!o.is_empty()),
            serde_json::Value::Null => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_basic_evaluate_with_context() {
        let mut context = HashMap::new();
        
        context.insert("a".to_string(), serde_json::Value::Bool(true));
        context.insert("b".to_string(), serde_json::Value::Bool(false));


        #[cfg(feature = "logging")]
        let logger = Logger::default();

        let evaluator = Evaluator::new(context, #[cfg(feature = "logging")] logger);

        let expr1 = "a && b"; 
        let expr2 = "a || b";
        let expr3 = "a && !b";
        let expr4 = "a || !b";
        let expr5 = "a && b || a && !b";
        let res1 = evaluator.evaluate(&expr1).unwrap();
        let res2 = evaluator.evaluate(&expr2).unwrap();
        let res3 = evaluator.evaluate(&expr3).unwrap();
        let res4 = evaluator.evaluate(&expr4).unwrap();
        let res5 = evaluator.evaluate(&expr5).unwrap();

        assert_eq!(res1, false);
        assert_eq!(res2, true);
        assert_eq!(res3, true);
        assert_eq!(res4, true);
        assert_eq!(res5, true);
    }

}

