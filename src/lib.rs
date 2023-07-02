use rslint_parser::{
    ast::{BinExpr, BinOp, CondExpr, DotExpr, Expr, Name, NameRef, UnaryExpr, UnaryOp},
    parse_text, AstNode, SyntaxKind, SyntaxNode, SyntaxNodeExt,
};

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Evaluation error")]
pub struct EvaluationError {
    #[from]
    source: NodeError,
}

#[derive(Error, Debug)]
#[error("Node error {message}, node: {node:?}")]
pub struct NodeError {
    message: String,
    node: Option<SyntaxNode>,
}

#[cfg(feature = "logging")]
use scribe_rust::Logger;
use serde_json::Value;

#[cfg(feature = "logging")]
use std::sync::Arc;

use std::collections::HashMap;

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

    pub fn evaluate(&self, expression: &str) -> Result<bool> {
        let ast = parse_text(expression, 0).syntax();
        let untyped_expr_node = match ast.first_child() {
            Some(node) => node,
            None => {
                return Err(NodeError {
                    message: "Empty expression".to_string(),
                    node: None,
                }
                .into())
            }
        };

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

    fn evaluate_node(&self, node: &SyntaxNode) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluting NodeKind: {:#?}, {:?}",
            node.kind(),
            node.to_string()
        ));

        let res = match node.kind() {
            SyntaxKind::EXPR_STMT => {
                let expr = match node.first_child() {
                    Some(node) => node,
                    None => {
                        return Err(NodeError {
                            message: "[Empty expression]".to_string(),
                            node: None,
                        }
                        .into())
                    }
                };
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
            res.as_ref()
        ));

        res
    }

    fn evaluate_bin_expr(&self, bin_expr: &BinExpr) -> Result<bool, NodeError> {
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

    fn evaluate_prefix_expr(&self, prefix_expr: &UnaryExpr) -> Result<bool, NodeError> {
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

    fn evaluate_cond_expr(&self, cond_expr: &CondExpr) -> Result<bool, NodeError> {
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

    fn evaluate_dot_expr(&self, dot_expr: &DotExpr) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Dot Expression: {:#?}",
            dot_expr.to_string()
        ));
        let mut left = dot_expr.clone().syntax().clone();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("DotExpr left {}", left.to_string()));

        while let Some(child) = left.child_with_kind(SyntaxKind::DOT_EXPR) {
            let dot_expr = match child.try_to::<DotExpr>() {
                Some(d) => d,
                None => {
                    return Err(NodeError {
                        message: "[DotExpr child is not a DotExpr]".to_string(),
                        node: Some(child),
                    })
                }
            };
            #[cfg(feature = "logging")]
            self.logger
                .trace(&format!("DotExpr child_expr {}", dot_expr.to_string()));
            left = dot_expr.clone().syntax().clone();
        }

        self.evaluate_by_name(left.first_token().unwrap().to_string())
    }

    fn evaluate_by_name(&self, identifier_name: String) -> Result<bool, NodeError> {
        let identifier_value = self.context.get(&identifier_name);

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Identifier Value: {:#?}", identifier_value));

        let res = match identifier_value {
            Some(serde_json::Value::Bool(b)) => Ok(*b),
            Some(serde_json::Value::String(s)) => {
                if s.contains('{') && s.contains('}') {
                    match serde_json::from_str::<Value>(s) {
                        Ok(v) => match v {
                            Value::Object(_) => Ok(true),
                            _ => Ok(false),
                        },
                        Err(_) => Ok(false),
                    }
                } else if s != ""
                    || s != "false"
                    || s != "0"
                    || s != "null"
                    || s != "undefined"
                    || s != "NaN"
                    || s != "Infinity"
                    || !s.is_empty()
                {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            Some(serde_json::Value::Number(n)) => Ok(n.as_i64().unwrap_or(0) != 0),
            Some(serde_json::Value::Null) => Ok(false),
            Some(serde_json::Value::Array(a)) => Ok(!a.is_empty()),
            Some(serde_json::Value::Object(_)) => Ok(true),
            None => Err(NodeError {
                message: "[Identifier Not Found In Context].".to_string(),
                node: None,
            }),
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Identifier Result: {:?}", res));
        res.map_err(|e| NodeError {
            message: format!("[Identifier Evaluation Error] => {}", e).to_string(),
            node: None,
        })
    }

    fn evaluate_name(&self, name: &Name) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Name: {:#?}", name.to_string()));
        let identifier_name = name.ident_token().unwrap().to_string();

        if identifier_name == "undefined"
            || identifier_name == "NaN"
            || identifier_name == "Infinity"
            || identifier_name == "null"
            || identifier_name == ""
        {
            return Ok(false);
        }

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_name_ref(&self, name_ref: &NameRef) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Name Reference: {:#?}",
            name_ref.to_string()
        ));
        let identifier_name = name_ref.ident_token().unwrap().to_string();

        if identifier_name == "undefined"
            || identifier_name == "NaN"
            || identifier_name == "Infinity"
            || identifier_name == "null"
            || identifier_name == ""
        {
            return Ok(false);
        }

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_identifier(&self, identifier: &Expr) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Identifier: {:#?}",
            identifier.to_string()
        ));
        let identifier_name = identifier.to_string();

        if identifier_name == "undefined"
            || identifier_name == "NaN"
            || identifier_name == "Infinity"
            || identifier_name == "null"
            || identifier_name == ""
        {
            return Ok(false);
        }

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_literal(&self, literal: &Expr) -> Result<bool, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Literal: {:#?}", literal.to_string()));

        let literal_value = literal.to_string();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Literal value: {:#?}", literal_value));

        let value: serde_json::Value =
            serde_json::from_str(&literal_value).map_err(|e| NodeError {
                message: format!("[Literal Evaluation Error] => {}", e).to_string(),
                node: Some(literal.syntax().clone()),
            })?;

        match value {
            serde_json::Value::Bool(b) => Ok(b),
            serde_json::Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
            serde_json::Value::String(s) => Ok(!s.is_empty()),
            serde_json::Value::Array(a) => Ok(!a.is_empty()),
            serde_json::Value::Object(o) => Ok(!o.is_empty()),
            serde_json::Value::Null => Ok(false),
        }
    }
}
