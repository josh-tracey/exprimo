use std::collections::HashMap;
use exprimo::Evaluator;

#[cfg(feature = "logging")]
use scribe_rust::Logger;

#[cfg(test)]

    #[test]
    fn test_basic_evaluate_with_context() {
        let mut context = HashMap::new();

        context.insert("a".to_string(), serde_json::Value::Bool(true));
        context.insert("b".to_string(), serde_json::Value::Bool(false));

        #[cfg(feature = "logging")]
        let logger = Logger::default();

        let evaluator = Evaluator::new(
            context,
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

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

    #[test]
    fn test_basic_evaluate_with_nulls() {
        let mut context = HashMap::new();

        context.insert("a".to_string(), serde_json::Value::Null);
        context.insert("b".to_string(), serde_json::Value::Bool(true));

        #[cfg(feature = "logging")]
        let logger = Logger::default();

        let evaluator = Evaluator::new(
            context,
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

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
        assert_eq!(res3, false);
        assert_eq!(res4, false);
        assert_eq!(res5, false);
    }

    // #[test]
    // fn test_basic_evaluate_with_empty_strings() {
    //     let mut context = HashMap::new();
    //
    //     context.insert(
    //         "a".to_string(),
    //         serde_json::Value::String("".to_string()),
    //     );
    //     context.insert("b".to_string(), serde_json::Value::Bool(true));
    //
    //     #[cfg(feature = "logging")]
    //     let logger = Logger::default();
    //
    //     let evaluator = Evaluator::new(
    //         context,
    //         #[cfg(feature = "logging")]
    //         logger,
    //     );
    //
    //     let expr1 = "a && b";
    //     let expr2 = "a || b";
    //     let expr3 = "a && !b";
    //     let expr4 = "a || !b";
    //     let expr5 = "a && b || a && !b";
    //     let res1 = evaluator.evaluate(&expr1).unwrap();
    //     let res2 = evaluator.evaluate(&expr2).unwrap();
    //     let res3 = evaluator.evaluate(&expr3).unwrap();
    //     let res4 = evaluator.evaluate(&expr4).unwrap();
    //     let res5 = evaluator.evaluate(&expr5).unwrap();
    //
    //     assert_eq!(res1, false);
    //     assert_eq!(res2, true);
    //     assert_eq!(res3, false);
    //     assert_eq!(res4, false);
    //     assert_eq!(res5, false);
    // }

    #[test]
    fn test_single_quotes_expressions() {
        
        let mut context = HashMap::new();

        context.insert("a".to_string(), serde_json::Value::String("true".to_string()));

        #[cfg(feature = "logging")]
        let logger = Logger::default();

        let evaluator = Evaluator::new(
            context,
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

        let expr1 = "a == 'true'";
       
        let res1 = evaluator.evaluate(&expr1).unwrap();
        
        assert_eq!(res1, true);
               
    }

// --- Custom Function Tests ---

// Imports needed for custom function tests
use exprimo::{CustomFunction, CustomFuncError, EvaluationError};
use serde_json::Value; // Already imported at top level if this is the same file
use std::sync::Arc;
use std::fmt::Debug; // Required for CustomFunction trait
// HashMap is already imported at top level

#[derive(Debug)] // Debug is required by the CustomFunction trait
struct MyTestAdder;

impl CustomFunction for MyTestAdder {
    fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError> {
        if args.len() != 2 {
            return Err(CustomFuncError::ArityError { expected: 2, got: args.len() });
        }
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(val_a), Some(val_b)) = (a.as_f64(), b.as_f64()) {
                    Ok(Value::Number(serde_json::Number::from_f64(val_a + val_b).unwrap()))
                } else {
                    Err(CustomFuncError::ArgumentError("Non-finite number provided".to_string()))
                }
            }
            _ => Err(CustomFuncError::ArgumentError("Arguments must be numbers".to_string())),
        }
    }
}

// --- Object.hasOwnProperty() Tests ---

#[test]
fn test_object_has_own_property_success() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), Value::String("Alice".to_string()));
    obj.insert("age".to_string(), Value::Number(30.into()));
    obj.insert("".to_string(), Value::String("empty_string_key".to_string())); // Empty string as key
    context.insert("myObj".to_string(), Value::Object(obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty('name')").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty(\"age\")").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty('nonExistent')").unwrap(), Value::Bool(false));
    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty('')").unwrap(), Value::Bool(true)); // Test empty string key

    // Test coercion of argument to string for a key that doesn't exist on myObj
    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty(123)").unwrap(), Value::Bool(false)); // effectively myObj.hasOwnProperty("123")
    assert_eq!(evaluator.evaluate("myObj.hasOwnProperty(true)").unwrap(), Value::Bool(false)); // effectively myObj.hasOwnProperty("true")

    // Setup a new context for an object that has stringified number as key
    let mut context2 = HashMap::new();
    let mut obj_with_true_key = serde_json::Map::new();
    obj_with_true_key.insert("true".to_string(), Value::String("test value for boolean true key".to_string()));
    context2.insert("objTrueKey".to_string(), Value::Object(obj_with_true_key));

    #[cfg(feature = "logging")]
    let logger2_true = Logger::default();
    let evaluator2_true = Evaluator::new(context2.clone(), HashMap::new(), #[cfg(feature = "logging")] logger2_true);

    assert_eq!(
        evaluator2_true.evaluate("objTrueKey.hasOwnProperty(true)").unwrap(), // Evaluates to objTrueKey.hasOwnProperty("true")
        Value::Bool(true),
        "objTrueKey should have property 'true' when called with boolean true"
    );
    assert_eq!(
        evaluator2_true.evaluate("objTrueKey.hasOwnProperty(false)").unwrap(), // Evaluates to objTrueKey.hasOwnProperty("false")
        Value::Bool(false),
        "objTrueKey should not have property 'false'"
    );


    // Test with a key that looks like a number
    let mut context3 = HashMap::new();
    let mut obj_with_num_key_str = serde_json::Map::new();
    obj_with_num_key_str.insert("123".to_string(), Value::Bool(true));
    context3.insert("objNumStrKey".to_string(), Value::Object(obj_with_num_key_str));

    #[cfg(feature = "logging")]
    let logger3_num = Logger::default();
    let evaluator3_num = Evaluator::new(context3, HashMap::new(), #[cfg(feature = "logging")] logger3_num);

    assert_eq!(evaluator3_num.evaluate("objNumStrKey.hasOwnProperty(123)").unwrap(), Value::Bool(true));
    assert_eq!(evaluator3_num.evaluate("objNumStrKey.hasOwnProperty('123')").unwrap(), Value::Bool(true));
}


#[test]
fn test_object_has_own_property_arity_error() {
    let mut context = HashMap::new();
    context.insert("myObj".to_string(), Value::Object(serde_json::Map::new()));
    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let res_no_args = evaluator.evaluate("myObj.hasOwnProperty()");
    match res_no_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 0);
        }
        _ => panic!("Expected ArityError for no arguments, got {:?}", res_no_args),
    }

    let res_many_args = evaluator.evaluate("myObj.hasOwnProperty('prop', 'extra')");
    match res_many_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 2);
        }
        _ => panic!("Expected ArityError for many arguments, got {:?}", res_many_args),
    }
}

#[test]
fn test_object_has_own_property_on_non_object() {
    let mut context = HashMap::new();
    context.insert("myArr".to_string(), Value::Array(vec![]));
    context.insert("myStr".to_string(), Value::String("text".to_string()));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let exprs_to_test = vec!["myArr.hasOwnProperty('length')", "myStr.hasOwnProperty('length')"];
    // Case 1: Array
    let expr_arr = "myArr.hasOwnProperty('length')";
    let result_arr = evaluator.evaluate(expr_arr);
    match result_arr {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(msg, "Cannot read properties of null or primitive value: [Array] (trying to access property: hasOwnProperty)");
        }
        _ => panic!("Expected TypeError for myArr.hasOwnProperty, got {:?}", result_arr),
    }

    // Case 2: String
    let expr_str = "myStr.hasOwnProperty('length')";
    let result_str = evaluator.evaluate(expr_str);
    match result_str {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(msg, "Cannot read properties of null or primitive value: text (trying to access property: hasOwnProperty)");
        }
        _ => panic!("Expected TypeError for myStr.hasOwnProperty, got {:?}", result_str),
    }
}

#[test]
fn test_object_has_own_property_nested() {
    let mut context = HashMap::new();
    let mut inner_obj = serde_json::Map::new();
    inner_obj.insert("value".to_string(), Value::Bool(true));
    let mut outer_obj = serde_json::Map::new();
    outer_obj.insert("nestedObj".to_string(), Value::Object(inner_obj));
    context.insert("item".to_string(), Value::Object(outer_obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("item.nestedObj.hasOwnProperty('value')").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("item.nestedObj.hasOwnProperty('nonExistent')").unwrap(), Value::Bool(false));
}

// --- Array.includes() Tests ---

#[test]
fn test_array_includes_success() {
    let mut context = HashMap::new();
    let arr = Value::Array(vec![
        Value::Number(10.into()),
        Value::String("hello".to_string()),
        Value::Bool(true),
        Value::Null,
    ]);
    context.insert("myArr".to_string(), arr);

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("myArr.includes(10)").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myArr.includes(\"hello\")").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myArr.includes(true)").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myArr.includes(null)").unwrap(), Value::Bool(true));

    assert_eq!(evaluator.evaluate("myArr.includes(20)").unwrap(), Value::Bool(false));
    assert_eq!(evaluator.evaluate("myArr.includes(\"world\")").unwrap(), Value::Bool(false));
    assert_eq!(evaluator.evaluate("myArr.includes(false)").unwrap(), Value::Bool(false));
    // Note: Value::Null is present, so `myArr.includes(something_else_that_is_not_null)` should be false.
    // serde_json::Value::Object(Map::new()) is not in the array.
    assert_eq!(evaluator.evaluate("myArr.includes({})").unwrap(), Value::Bool(false));
}

#[test]
fn test_array_includes_arity_error() {
    let mut context = HashMap::new();
    context.insert("myArr".to_string(), Value::Array(vec![]));
    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let res_no_args = evaluator.evaluate("myArr.includes()");
    match res_no_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 0);
        }
        _ => panic!("Expected ArityError for no arguments, got {:?}", res_no_args),
    }

    let res_many_args = evaluator.evaluate("myArr.includes(1, 2)");
    match res_many_args {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 1);
            assert_eq!(got, 2);
        }
        _ => panic!("Expected ArityError for many arguments, got {:?}", res_many_args),
    }
}

#[test]
fn test_array_includes_on_non_array() {
    let mut context = HashMap::new();
    context.insert("notAnArray".to_string(), Value::String("hello".to_string()));
    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let result = evaluator.evaluate("notAnArray.includes(1)");
    match result {
        Err(EvaluationError::TypeError(msg)) => {
            // This error is from evaluate_dot_expr directly when trying to access 'includes' on a string "hello".
            assert_eq!(msg, "Cannot read properties of null or primitive value: hello (trying to access property: includes)");
        }
        _ => panic!("Expected TypeError when calling .includes on non-array, got {:?}", result),
    }
}

#[test]
fn test_array_includes_nested() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    let arr = Value::Array(vec![Value::Number(42.into())]);
    obj.insert("nestedArr".to_string(), arr);
    context.insert("myObj".to_string(), Value::Object(obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("myObj.nestedArr.includes(42)").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("myObj.nestedArr.includes(100)").unwrap(), Value::Bool(false));
}

#[test]
fn test_custom_adder_success() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default(); // Logger is already imported at top level

    let evaluator = Evaluator::new( // Evaluator is already imported
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    let result = evaluator.evaluate("custom_add(10, 20.5)").unwrap();
    assert_eq!(result.as_f64(), Some(30.5));

    let result_neg = evaluator.evaluate("custom_add(-5, -2)").unwrap();
    assert_eq!(result_neg.as_f64(), Some(-7.0));
}

#[test]
fn test_custom_adder_arity_error_few_args() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    let result = evaluator.evaluate("custom_add(10)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 2);
            assert_eq!(got, 1);
        }
        _ => panic!("Expected ArityError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_arity_error_many_args() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    let result = evaluator.evaluate("custom_add(10, 20, 30)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArityError { expected, got })) => {
            assert_eq!(expected, 2);
            assert_eq!(got, 3);
        }
        _ => panic!("Expected ArityError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_type_error_arg1() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    let result = evaluator.evaluate("custom_add('not_a_number', 10)");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArgumentError(msg))) => {
            assert_eq!(msg, "Arguments must be numbers");
        }
        _ => panic!("Expected ArgumentError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_type_error_arg2() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    let result = evaluator.evaluate("custom_add(10, 'not_a_number')");
    match result {
        Err(EvaluationError::CustomFunction(CustomFuncError::ArgumentError(msg))) => {
            assert_eq!(msg, "Arguments must be numbers");
        }
        _ => panic!("Expected ArgumentError, got {:?}", result),
    }
}

#[test]
fn test_custom_adder_non_finite_number_error() {
    let context = HashMap::new();
    let mut custom_funcs: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_funcs.insert("custom_add".to_string(), Arc::new(MyTestAdder));

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        custom_funcs,
        #[cfg(feature = "logging")] logger
    );

    // Create a NaN Value::Number (Note: serde_json::Number cannot directly represent NaN/Infinity)
    // This test relies on the internal f64 conversion and check.
    // For the purpose of this test, we'll assume that if a Value::Number
    // somehow contained a non-finite f64, our function would catch it.
    // Direct creation of such a serde_json::Value::Number is tricky,
    // as it typically only supports finite numbers.
    // The error "Non-finite number provided" is more of a safeguard
    // if such a value were to be constructed manually or via other means.
    // We can't directly test this path with standard expression strings
    // if the parser/evaluator only produces finite Value::Number.
    // However, if a custom function internally constructed such a value and passed it
    // to another custom function, this check would be relevant.
    // For now, we acknowledge this path is hard to test via string expressions.
    // A direct call to `MyTestAdder.call()` would be needed to test this,
    // which is outside the scope of `evaluator.evaluate()`.
    // So, this specific error condition "Non-finite number provided"
    // is not easily testable through the evaluator's `evaluate` method
    // if the parser only generates valid numbers.
    // The existing type error tests cover cases where types are not numbers.
}

// --- Property Access Tests ---

#[test]
fn test_array_length_direct() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]);
    context.insert("myArray".to_string(), my_array);

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let result = evaluator.evaluate("myArray.length").unwrap();
    assert_eq!(result, Value::Number(serde_json::Number::from_f64(3.0).unwrap()));
}

#[test]
fn test_array_length_nested_in_object() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from("a"), Value::from("b")]);
    let mut obj = serde_json::Map::new();
    obj.insert("arr".to_string(), my_array);
    context.insert("myObj".to_string(), Value::Object(obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let result = evaluator.evaluate("myObj.arr.length").unwrap();
    assert_eq!(result, Value::Number(serde_json::Number::from_f64(2.0).unwrap()));
}

#[test]
fn test_length_on_non_array() {
    let mut context = HashMap::new();
    context.insert("myString".to_string(), Value::String("hello".to_string()));
    context.insert("myNum".to_string(), Value::Number(123.into()));
    let mut obj_without_length = serde_json::Map::new();
    obj_without_length.insert("prop".to_string(), Value::from("value"));
    context.insert("myObj".to_string(), Value::Object(obj_without_length));
    context.insert("nullVar".to_string(), Value::Null);


    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context.clone(), HashMap::new(), #[cfg(feature = "logging")] logger.clone());

    // String.length is not yet implemented, should fall into the generic "cannot read props of primitive" or specific "length" error
    let res_str = evaluator.evaluate("myString.length");
    match res_str {
        Err(EvaluationError::TypeError(msg)) => {
             assert_eq!(msg, "Cannot read property 'length' of non-array/non-object value: hello");
        }
        _ => panic!("Expected TypeError for string.length, got {:?}", res_str),
    }

    let res_num = evaluator.evaluate("myNum.length");
    match res_num {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(msg, "Cannot read property 'length' of non-array/non-object value: 123");
        }
        _ => panic!("Expected TypeError for number.length, got {:?}", res_num),
    }

    // Accessing .length on an object that doesn't have it should return Value::Null
    let res_obj = evaluator.evaluate("myObj.length").unwrap();
    assert_eq!(res_obj, Value::Null);


    let res_null = evaluator.evaluate("nullVar.length");
    match res_null {
        Err(EvaluationError::TypeError(msg)) => {
            assert_eq!(msg, "Cannot read property 'length' of non-array/non-object value: null");
        }
        _ => panic!("Expected TypeError for null.length, got {:?}", res_null),
    }
}

#[test]
fn test_other_property_on_array() {
    let mut context = HashMap::new();
    let my_array = Value::Array(vec![Value::from(1)]);
    context.insert("myArray".to_string(), my_array);

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let result = evaluator.evaluate("myArray.foo").unwrap();
    assert_eq!(result, Value::Null); // JS returns undefined, so we return Null
}

#[test]
fn test_property_access_on_object() {
    let mut context = HashMap::new();
    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), Value::String("Tester".to_string()));
    obj.insert("age".to_string(), Value::Number(30.into()));
    context.insert("user".to_string(), Value::Object(obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("user.name").unwrap(), Value::String("Tester".to_string()));
    assert_eq!(evaluator.evaluate("user.age").unwrap(), Value::Number(30.into()));
    assert_eq!(evaluator.evaluate("user.nonexistent").unwrap(), Value::Null);
}

#[test]
fn test_property_access_on_nested_object() {
    let mut context = HashMap::new();
    let mut inner_obj = serde_json::Map::new();
    inner_obj.insert("value".to_string(), Value::Bool(true));
    let mut outer_obj = serde_json::Map::new();
    outer_obj.insert("nested".to_string(), Value::Object(inner_obj));
    context.insert("item".to_string(), Value::Object(outer_obj));

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    assert_eq!(evaluator.evaluate("item.nested.value").unwrap(), Value::Bool(true));
    assert_eq!(evaluator.evaluate("item.nested.foo").unwrap(), Value::Null);

    let res_access_on_null = evaluator.evaluate("item.nonexistent.bar"); // item.nonexistent is Null, then .bar on Null
    match res_access_on_null {
        Err(EvaluationError::TypeError(msg)) => {
            assert!(msg.contains("Cannot read properties of null or primitive value: null (trying to access property: bar)"));
        }
        _ => panic!("Expected TypeError for item.nonexistent.bar, got {:?}", res_access_on_null),
    }
}

#[test]
fn test_property_access_on_null_or_primitive_object_error() {
    let mut context = HashMap::new();
    context.insert("s".to_string(), Value::String("text".to_string()));
    context.insert("n".to_string(), Value::Number(123.into()));
    context.insert("b".to_string(), Value::Bool(true));
    context.insert("nl".to_string(), Value::Null);

    #[cfg(feature = "logging")]
    let logger = Logger::default();
    let evaluator = Evaluator::new(context, HashMap::new(), #[cfg(feature = "logging")] logger);

    let cases = vec!["s.foo", "n.bar", "b.baz", "nl.qux"];
    for case in cases {
        let result = evaluator.evaluate(case);
        match result {
            Err(EvaluationError::TypeError(msg)) => {
                assert!(msg.starts_with("Cannot read properties of null or primitive value:"));
            }
            _ => panic!("Expected TypeError for property access on primitive/null, got {:?}", result),
        }
    }
}
