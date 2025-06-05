use std::collections::HashMap;
use std::fmt::Debug; // For CustomFunction trait
use std::sync::Arc;   // For Arc<dyn CustomFunction>
use serde_json::Value;
use thiserror::Error;
use rslint_parser::{
    ast::{BinExpr, BinOp, CondExpr, DotExpr, Expr, Name, NameRef, UnaryExpr, UnaryOp, CallExpr}, // Removed ExprOrSpread
    parse_text, AstNode, SyntaxKind, SyntaxNode,
};
#[cfg(feature = "logging")]
use scribe_rust::Logger; // Assuming this is the correct path to Logger

#[derive(Error, Debug)]
pub enum CustomFuncError {
    #[error("Argument error: {0}")]
    ArgumentError(String),
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    ArityError { expected: usize, got: usize },
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Node evaluation failed: {0}")]
    Node(#[from] NodeError),
    #[error("Custom function execution failed: {0}")]
    CustomFunction(#[from] CustomFuncError),
    #[error("Type error: {0}")]
    TypeError(String),
}

#[derive(Error, Debug)]
#[error("Node error {message}, node: {node:?}")]
pub struct NodeError {
    message: String,
    node: Option<SyntaxNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInMethodKind {
    ArrayIncludes,
    ObjectHasOwnProperty, // Added
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvableValue {
    Json(Value),
    BuiltInMethod {
        object: Box<Value>, // The object on which the method is called (e.g., the array)
        method: BuiltInMethodKind,
    },
}

impl ResolvableValue {
    fn try_into_value(self) -> Result<Value, EvaluationError> {
        match self {
            ResolvableValue::Json(val) => Ok(val),
            ResolvableValue::BuiltInMethod { object, method } => Err(EvaluationError::TypeError(format!(
                "Cannot use built-in method {:?} on {:?} as a value.", // Adjusted error message
                method, object
            ))),
        }
    }
}

pub trait CustomFunction: Debug + Send + Sync {
    fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError>;
}

pub struct Evaluator {
    context: HashMap<String, Value>,
    custom_functions: HashMap<String, Arc<dyn CustomFunction>>,
    #[cfg(feature = "logging")]
    logger: Arc<Logger>, // Ensure Logger is the correct type from scribe_rust
}

impl Evaluator {
    pub fn new(
        context: HashMap<String, Value>,
        custom_functions: HashMap<String, Arc<dyn CustomFunction>>,
        #[cfg(feature = "logging")] logger: Arc<Logger>, // Ensure Logger is the correct type
    ) -> Self {
        Evaluator {
            context,
            custom_functions,
            #[cfg(feature = "logging")]
            logger,
        }
    }

    pub fn evaluate(&self, expression: &str) -> Result<Value, EvaluationError> {
        let ast = parse_text(expression, 0).syntax();
        let untyped_expr_node = match ast.first_child() {
            Some(node) => node,
            None => {
                return Err(EvaluationError::Node(NodeError {
                    message: "Empty expression".to_string(),
                    node: None,
                }));
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

    fn evaluate_node(&self, node: &SyntaxNode) -> Result<Value, EvaluationError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating NodeKind: {:#?}, {:?}",
            node.kind(),
            node.to_string()
        ));

        let res = match node.kind() {
            SyntaxKind::EXPR_STMT => {
                let expr = node.first_child().ok_or_else(|| EvaluationError::Node(NodeError {
                    message: "[Empty expression]".to_string(),
                    node: None,
                }))?;
                self.evaluate_node(&expr)
            }
            SyntaxKind::DOT_EXPR => self.evaluate_dot_expr(&DotExpr::cast(node.clone()).unwrap())?.try_into_value(),
            SyntaxKind::NAME_REF => self.evaluate_name_ref(&NameRef::cast(node.clone()).unwrap()).map_err(EvaluationError::from),
            SyntaxKind::NAME => self.evaluate_name(&Name::cast(node.clone()).unwrap()).map_err(EvaluationError::from),
            SyntaxKind::BIN_EXPR => self.evaluate_bin_expr(&BinExpr::cast(node.clone()).unwrap()).map_err(EvaluationError::from),
            SyntaxKind::LITERAL => self.evaluate_literal(&Expr::cast(node.clone()).unwrap()).map_err(EvaluationError::from),
            SyntaxKind::COND_EXPR => {
                self.evaluate_cond_expr(&CondExpr::cast(node.clone()).unwrap()).map_err(EvaluationError::from)
            }
            SyntaxKind::IDENT => self.evaluate_identifier(&Expr::cast(node.clone()).unwrap()).map_err(EvaluationError::from),
            SyntaxKind::UNARY_EXPR => {
                self.evaluate_prefix_expr(&UnaryExpr::cast(node.clone()).unwrap()).map_err(EvaluationError::from)
            }
            SyntaxKind::CALL_EXPR => {
                self.evaluate_call_expr(&CallExpr::cast(node.clone()).unwrap())
            }
            // Handle simple array and object literals
            SyntaxKind::ARRAY_EXPR => {
                // For now, only support empty array literal []
                // Complex array literals [1,2,3] would require iterating elements
                if node.children().count() == 0 {
                    Ok(Value::Array(vec![]))
                } else {
                    Err(EvaluationError::Node(NodeError {
                        message: "Complex array literals are not yet supported.".to_string(),
                        node: Some(node.clone()),
                    }))
                }
            }
            SyntaxKind::OBJECT_EXPR => {
                // For now, only support empty object literal {}
                // Complex object literals {a:1} would require parsing properties
                if node.children().count() == 0 {
                    Ok(Value::Object(serde_json::Map::new()))
                } else {
                    Err(EvaluationError::Node(NodeError {
                        message: "Complex object literals are not yet supported.".to_string(),
                        node: Some(node.clone()),
                    }))
                }
            }
            _ => Err(EvaluationError::Node(NodeError {
                message: format!("Unsupported syntax kind: {:?}", node.kind()),
                node: Some(node.clone()),
            })),
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "NodeKind: {:?} => {:#?}",
            node.kind(),
            res.as_ref()
        ));

        res
    }

    fn evaluate_bin_expr(&self, bin_expr: &BinExpr) -> Result<Value, EvaluationError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Binary Expression: {:#?}",
            bin_expr.to_string()
        ));

        let left = bin_expr.lhs().ok_or_else(|| NodeError {
            message: "[Empty BinExpr Left Expression]".to_string(),
            node: Some(bin_expr.syntax().clone()),
        })?;
        let right = bin_expr.rhs().ok_or_else(|| NodeError {
            message: "[Empty BinExpr Right Expression]".to_string(),
            node: Some(bin_expr.syntax().clone()),
        })?;

        let left_value = self.evaluate_node(left.syntax())?;
        let right_value = self.evaluate_node(right.syntax())?;

        let op = bin_expr.op_details();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp left_value {:?}", left_value));

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp right_value {:?}", right_value));

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("BinaryOp op_details {:?}", op));

        let result = match op {
            Some((_, BinOp::Plus)) => self.add_values(left_value, right_value),
            Some((_, BinOp::Minus)) => self.subtract_values(left_value, right_value),
            Some((_, BinOp::Times)) => self.multiply_values(left_value, right_value),
            Some((_, BinOp::Divide)) => self.divide_values(left_value, right_value),
            Some((_, BinOp::Remainder)) => self.modulo_values(left_value, right_value),
            Some((_, BinOp::LogicalAnd)) => Ok(Value::Bool(
                self.to_boolean(&left_value)? && self.to_boolean(&right_value)?
            )),
            Some((_, BinOp::LogicalOr)) => Ok(Value::Bool(
                self.to_boolean(&left_value)? || self.to_boolean(&right_value)?
            )),
            Some((_, BinOp::Equality)) | Some((_, BinOp::StrictEquality)) => Ok(Value::Bool(
                self.abstract_equality(&left_value, &right_value),
            )),
            Some((_, BinOp::Inequality)) | Some((_, BinOp::StrictInequality)) => Ok(Value::Bool(
                !self.abstract_equality(&left_value, &right_value),
            )),
            Some((_, BinOp::GreaterThan)) => {
                self.compare_values(&left_value, &right_value, |a, b| a > b)
            }
            Some((_, BinOp::LessThan)) => {
                self.compare_values(&left_value, &right_value, |a, b| a < b)
            }
            Some((_, BinOp::GreaterThanOrEqual)) => {
                self.compare_values(&left_value, &right_value, |a, b| a >= b)
            }
            Some((_, BinOp::LessThanOrEqual)) => {
                self.compare_values(&left_value, &right_value, |a, b| a <= b)
            }
            _ => Err(EvaluationError::Node(NodeError {
                message: "Unsupported binary operator".to_string(),
                node: Some(bin_expr.syntax().clone()),
            })),
        }?;

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Binary Result: {:?}", result));

        Ok(result)
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, EvaluationError> {
        match (left.clone(), right.clone()) {
            (Value::Number(l), Value::Number(r)) => {
                let sum = l.as_f64().unwrap() + r.as_f64().unwrap();
                Ok(Value::Number(serde_json::Number::from_f64(sum).unwrap()))
            }
            (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
            (Value::String(l), r) => Ok(Value::String(l + &self.value_to_string(&r))),
            (l, Value::String(r)) => Ok(Value::String(self.value_to_string(&l) + &r)),
            _ => {
                // Type coercion similar to JavaScript
                // This branch might need to use to_number if we want it to behave like JS '+' with mixed types that coerce to number first.
                // However, current implementation coerces to string.
                // If numeric conversion is desired for non-string/non-number types,
                // to_number should be used, and it returns EvaluationError.
                // For now, sticking to string concatenation for non-numeric types.
                let l_str = self.value_to_string(&left);
                let r_str = self.value_to_string(&right);
                Ok(Value::String(l_str + &r_str))
            }
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, EvaluationError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num - r_num).unwrap(),
        ))
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, EvaluationError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num * r_num).unwrap(),
        ))
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value, EvaluationError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        if r_num == 0.0 {
            return Err(EvaluationError::Node(NodeError {
                message: "Division by zero".to_string(),
                node: None,
            }));
        }
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num / r_num).unwrap(),
        ))
    }

    fn modulo_values(&self, left: Value, right: Value) -> Result<Value, EvaluationError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num % r_num).unwrap(),
        ))
    }

    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> Result<Value, EvaluationError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let l_num = self.to_number(left)?;
        let r_num = self.to_number(right)?;
        Ok(Value::Bool(cmp(l_num, r_num)))
    }

    fn evaluate_prefix_expr(&self, prefix_expr: &UnaryExpr) -> Result<Value, EvaluationError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Prefix Expression: {:#?}",
            prefix_expr.to_string()
        ));
        let expr = prefix_expr.expr().ok_or_else(|| NodeError {
            message: "[Empty PrefixExpr Expression]".to_string(),
            node: Some(prefix_expr.syntax().clone()),
        })?;
        let expr_value = self.evaluate_node(expr.syntax())?;

        let op = prefix_expr.op_details();

        let result = match op {
            Some((_, UnaryOp::LogicalNot)) => Value::Bool(!self.to_boolean(&expr_value)?),
            Some((_, UnaryOp::Minus)) => {
                let num = self.to_number(&expr_value)?;
                Value::Number(serde_json::Number::from_f64(-num).unwrap())
            }
            Some((_, UnaryOp::Plus)) => {
                let num = self.to_number(&expr_value)?;
                Value::Number(serde_json::Number::from_f64(num).unwrap())
            }
            _ => {
                return Err(EvaluationError::Node(NodeError {
                    message: "Unsupported unary operator".to_string(),
                    node: Some(prefix_expr.syntax().clone()),
                }))
            }
        };
        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Prefix Result: {:?}", result));

        Ok(result)
    }

    fn evaluate_cond_expr(&self, cond_expr: &CondExpr) -> Result<Value, EvaluationError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Conditional Expression: {:#?}",
            cond_expr.to_string()
        ));
        let cond = cond_expr.test().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Test Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;
        let true_expr = cond_expr.cons().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Consequent Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;
        let false_expr = cond_expr.alt().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Alternate Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;

        let cond_value = self.evaluate_node(cond.syntax())?; // Returns EvaluationError
        let cond_bool = self.to_boolean(&cond_value)?; // Returns EvaluationError

        let result = if cond_bool {
            self.evaluate_node(true_expr.syntax())? // Returns EvaluationError
        } else {
            self.evaluate_node(false_expr.syntax())? // Returns EvaluationError
        };

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Conditional Result: {:?}", result));

        Ok(result)
    }

    fn evaluate_dot_expr(&self, dot_expr: &DotExpr) -> Result<ResolvableValue, EvaluationError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Dot Expression: {:#?}", dot_expr));

        let object_expr = dot_expr.object().ok_or_else(|| EvaluationError::Node(NodeError {
            message: "Missing object in dot expression".to_string(),
            node: Some(dot_expr.syntax().clone()),
        }))?;

        let prop_name_ident = dot_expr.prop().ok_or_else(|| EvaluationError::Node(NodeError {
            message: "Missing property name in dot expression".to_string(),
            node: Some(dot_expr.syntax().clone()),
        }))?;
        // In rslint_parser, prop for DotExpr is an Name rather than NameRef or Ident
        // So we need to get its text representation.
        let prop_name = prop_name_ident.syntax().text().to_string();

        // Evaluate the object part of the dot expression
        let object_value = self.evaluate_node(object_expr.syntax())?;

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Dot Expression: object_value={:?}, prop_name='{}'",
            object_value, prop_name
        ));

        match object_value {
            Value::Array(arr) => {
                if prop_name == "length" {
                    Ok(ResolvableValue::Json(Value::Number(serde_json::Number::from_f64(arr.len() as f64).unwrap())))
                } else if prop_name == "includes" {
                    Ok(ResolvableValue::BuiltInMethod {
                        object: Box::new(Value::Array(arr.clone())), // Clone the array for the method context
                        method: BuiltInMethodKind::ArrayIncludes,
                    })
                } else {
                    // Accessing other properties like myArray.foo returns undefined in JS.
                    Ok(ResolvableValue::Json(Value::Null))
                }
            }
            Value::Object(map) => {
                if prop_name == "hasOwnProperty" {
                    Ok(ResolvableValue::BuiltInMethod {
                        object: Box::new(Value::Object(map.clone())), // Clone the object for the method context
                        method: BuiltInMethodKind::ObjectHasOwnProperty,
                    })
                } else {
                    Ok(ResolvableValue::Json(map.get(&prop_name).cloned().unwrap_or(Value::Null)))
                }
            }
            _ => {
                 if prop_name == "length" { // Check for .length on non-array/non-object first
                     Err(EvaluationError::TypeError(format!(
                        "Cannot read property 'length' of non-array/non-object value: {}", // Clarified error
                        self.value_to_string(&object_value)
                    )))
                 } else {
                    Err(EvaluationError::TypeError(format!(
                        "Cannot read properties of null or primitive value: {} (trying to access property: {})",
                        self.value_to_string(&object_value),
                        prop_name
                    )))
                 }
            }
        }
    }

    // Implement abstract equality similar to JavaScript
    fn abstract_equality(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Number(l), Value::Number(r)) => l.as_f64() == r.as_f64(),
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            _ => false,
        }
    }

    fn evaluate_by_name(&self, identifier_name: String) -> Result<Value, NodeError> {
        let identifier_value = self.context.get(&identifier_name);

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Identifier Value: {:#?}", identifier_value));

        match identifier_value {
            Some(value) => Ok(value.clone()),
            None => Err(NodeError {
                message: format!("Identifier '{}' not found in context.", identifier_name),
                node: None,
            }),
        }
    }

    fn evaluate_name(&self, name: &Name) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Name: {:#?}", name.to_string()));
        let identifier_name = name
            .ident_token()
            .ok_or_else(|| NodeError {
                message: "[Empty Name]".to_string(),
                node: Some(name.syntax().clone()),
            })?
            .to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_name_ref(&self, name_ref: &NameRef) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Name Reference: {:#?}",
            name_ref.to_string()
        ));
        let identifier_name = name_ref
            .ident_token()
            .ok_or_else(|| NodeError {
                message: "[Empty NameRef]".to_string(),
                node: Some(name_ref.syntax().clone()),
            })?
            .to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_identifier(&self, identifier: &Expr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Identifier: {:#?}",
            identifier.to_string()
        ));
        let identifier_name = identifier.to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_literal(&self, literal: &Expr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Literal: {:#?}", literal.to_string()));

        let literal_str = literal.to_string();

        // Handle numeric literals
        if let Ok(number) = literal_str.parse::<f64>() {
            return Ok(Value::Number(serde_json::Number::from_f64(number).unwrap()));
        }

        // Handle string literals
        if literal_str.starts_with('"') || literal_str.starts_with('\'') {
            let unquoted = literal_str
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            return Ok(Value::String(unquoted));
        }

        // Handle boolean literals
        match literal_str.as_str() {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "null" => return Ok(Value::Null),
            _ => {}
        }

        Err(NodeError {
            message: format!("Unknown literal type: {}", literal_str),
            node: Some(literal.syntax().clone()),
        })
    }

    fn to_number(&self, value: &Value) -> Result<f64, EvaluationError> {
        match value {
            Value::Number(n) => Ok(n.as_f64().unwrap()),
            Value::String(s) => s.parse::<f64>().map_err(|_| EvaluationError::Node(NodeError {
                message: format!("Cannot convert string '{}' to number", s),
                node: None,
            })),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Ok(0.0),
            _ => Err(EvaluationError::Node(NodeError {
                message: "Cannot convert value to number".to_string(),
                node: None,
            })),
        }
    }

    fn to_boolean(&self, value: &Value) -> Result<bool, EvaluationError> {
        let result = match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => {
                let num = n.as_f64().unwrap();
                num != 0.0 && !num.is_nan()
            }
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        };
        Ok(result)
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
        }
    }

    fn evaluate_call_expr(&self, call_expr: &CallExpr) -> Result<Value, EvaluationError> {
        let callee_expr_node = call_expr.callee().ok_or_else(|| EvaluationError::Node(NodeError {
            message: "Missing callee in call expression".to_string(),
            node: Some(call_expr.syntax().clone()),
        }))?;

        let callee_syntax = callee_expr_node.syntax();

        // Evaluate arguments first, as they are needed in both branches
        let mut evaluated_args = Vec::new();
        if let Some(arg_list_node) = call_expr.arguments() {
            for arg_expr in arg_list_node.args() {
                let arg_val = self.evaluate_node(arg_expr.syntax())?;
                evaluated_args.push(arg_val);
            }
        }

        match callee_syntax.kind() {
            SyntaxKind::NAME_REF => {
                // Handle custom functions (e.g., myFunc())
                let name_ref = NameRef::cast(callee_syntax.clone()).unwrap(); // Should be safe given kind check
                let func_name = name_ref.syntax().text().to_string();
                if let Some(func) = self.custom_functions.get(&func_name) {
                    func.call(&evaluated_args).map_err(EvaluationError::from)
                } else {
                    Err(EvaluationError::Node(NodeError {
                        message: format!("Function '{}' not found.", func_name),
                        node: Some(callee_syntax.clone()),
                    }))
                }
            }
            SyntaxKind::DOT_EXPR => {
                // Handle method calls (e.g., myArray.includes())
                let dot_expr = DotExpr::cast(callee_syntax.clone()).unwrap(); // Should be safe
                let resolvable_callee = self.evaluate_dot_expr(&dot_expr)?;

                match resolvable_callee {
                    ResolvableValue::BuiltInMethod { object, method } => {
                        match method {
                            BuiltInMethodKind::ArrayIncludes => {
                                if evaluated_args.len() != 1 {
                                    return Err(EvaluationError::CustomFunction(CustomFuncError::ArityError {
                                        expected: 1,
                                        got: evaluated_args.len(),
                                    }));
                                }
                                if let Value::Array(arr) = *object {
                                    let target_value = &evaluated_args[0];
                                    let mut found = false;
                                    for item in arr.iter() {
                                        if self.abstract_equality(item, target_value) {
                                            found = true;
                                            break;
                                        }
                                    }
                                    Ok(Value::Bool(found))
                                } else {
                                    // This case should ideally be prevented by how BuiltInMethod is constructed in evaluate_dot_expr
                                    Err(EvaluationError::TypeError("ArrayIncludes method called on a non-array internal object.".to_string()))
                                }
                            }
                            BuiltInMethodKind::ObjectHasOwnProperty => {
                                if evaluated_args.len() != 1 {
                                    return Err(EvaluationError::CustomFunction(CustomFuncError::ArityError {
                                        expected: 1,
                                        got: evaluated_args.len(),
                                    }));
                                }
                                let prop_key_val = &evaluated_args[0];
                                // Coerce argument to string, similar to JS
                                let prop_key_str = self.value_to_string(prop_key_val);

                                if let Value::Object(obj_map) = *object { // object is the Box<Value>
                                    Ok(Value::Bool(obj_map.contains_key(&prop_key_str)))
                                } else {
                                    // This should not happen if BuiltInMethod is constructed correctly
                                    Err(EvaluationError::TypeError("ObjectHasOwnProperty method called on a non-object internal object.".to_string()))
                                }
                            }
                        }
                    }
                    ResolvableValue::Json(json_val) => {
                        Err(EvaluationError::TypeError(format!(
                            "'{}' (resulting from expression '{}') is not a function.",
                            self.value_to_string(&json_val),
                            dot_expr.syntax().text()
                        )))
                    }
                }
            }
            _ => {
                Err(EvaluationError::Node(NodeError {
                    message: format!("Unsupported callee type: {:?}. Expected identifier or member expression.", callee_syntax.kind()),
                    node: Some(callee_syntax.clone()),
                }))
            }
        }
    }
}
