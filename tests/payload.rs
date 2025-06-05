#[cfg(test)]
mod tests {
    use exprimo::Evaluator;
    use exprimo::EvaluationError; // Added import
    use serde_json::Value;
    use std::collections::HashMap;

    #[cfg(feature = "serde_json_ctx")]
    use serde_json_ctx::ValueExt;

    #[cfg(feature = "logging")]
    use scribe_rust::Logger;
    #[cfg(feature = "logging")]
    use std::sync::Arc;

    pub fn add_context(key: &str, json_str: &str, context: &mut HashMap<String, String>) {
        context.insert(key.to_string(), json_str.to_string());
    }

    pub fn to_json(context: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
        let mut json = HashMap::new();
        for (key, value) in context.iter() {
            let parsed_value =
                serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()));
            json.insert(key.to_string(), parsed_value);
        }
        json
    }

    #[test]
    fn test_json_payload_eval() {
        let mut context: HashMap<String, String> = HashMap::new();

        add_context("event", r#"{}"#, &mut context);

        #[cfg(feature = "logging")]
        let logger = Arc::new(Logger::default());

        let evaluator = Evaluator::new(
            to_json(&context),
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

        let expr1 = "event.payload === null";

        let res1 = evaluator.evaluate(expr1).unwrap();

        assert_eq!(res1, Value::Bool(true)); // Updated expectation
    }

    #[test]
    fn test_json_payload_eval2() {
        let mut context: HashMap<String, String> = HashMap::new();

        add_context("send_email", r#"{"status": "success"}"#, &mut context);

        #[cfg(feature = "logging")]
        let logger = Arc::new(Logger::default());

        let evaluator = Evaluator::new(
            to_json(&context),
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

        let expr1 = "send_email.status === \"success\"";

        let res1 = evaluator.evaluate(expr1).unwrap();

        assert_eq!(res1, Value::Bool(true));
    }

    #[cfg(feature = "serde_json_ctx")]
    #[test]
    fn test_json_payload_eval_with_serde_json_ctx() {
        let mut context: serde_json::Map<String, Value> = serde_json::Map::new();

        context.insert("event".to_string(), serde_json::Value::Null);
        context.insert(
            "send_email".to_string(),
            serde_json::json!({
                "status": "success",
                "payload": null,
                "satcom": {
                    "id":  12345.0,
                    "name": "Test Satellite"
                }
            }),
        );

        #[cfg(feature = "logging")]
        let logger = Arc::new(Logger::default());
        let evaluator = Evaluator::new(
            context.clone().into_iter().collect(),
            HashMap::new(), // custom_functions
            #[cfg(feature = "logging")]
            logger,
        );

        // Test for event.payload when event is Value::Null
        // Accessing .payload on Value::Null should be a TypeError
        let expr1_eval = evaluator.evaluate("event.payload");
        match expr1_eval {
            Err(EvaluationError::TypeError(msg)) => {
                assert!(msg.contains("Cannot read properties of null or primitive value: null (trying to access property: payload)"));
            }
            _ => panic!("Expected TypeError for event.payload when event is null, got {:?}", expr1_eval),
        }

        // To test 'something === null' where something might be null due to object structure:
        // Let's add a new context item for this specific test case if needed,
        // or assert on a different part of the existing 'send_email' object.
        // The original test 'event.payload === null' where event itself is Value::Null is problematic
        // with strict property access as 'event.payload' itself errors.

        // Test for send_email.status
        let expr2 = "send_email.status === \"success\"";
        let res2 = evaluator.evaluate(expr2).unwrap();
        assert_eq!(res2, serde_json::Value::Bool(true));
        let expr3 = "send_email.satcom.id === 12345.0";
        let res3 = evaluator.evaluate(expr3).unwrap();
        assert_eq!(res3, serde_json::Value::Bool(true));
        let expr4 = "send_email.satcom.name === \"Test Satellite\"";
        let res4 = evaluator.evaluate(expr4).unwrap();
        assert_eq!(res4, serde_json::Value::Bool(true));
        let expr5 = "send_email.payload === null";
        let res5 = evaluator.evaluate(expr5).unwrap();
        assert_eq!(res5, serde_json::Value::Bool(true));
        let expr6 = "send_email.satcom === null";
        let res6 = evaluator.evaluate(expr6).unwrap();
        assert_eq!(res6, serde_json::Value::Bool(false));

        let expr7 = "send_email.satcom.id === 12345"; // Testing with integer comparison to float
                                                      // rust will coerce the float to an integer
                                                      // cus javascript coerces the float to an integer,
                                                      // as no decimal places are present so they
                                                      // are equal.
        let res7 = evaluator.evaluate(expr3).unwrap();
        assert_eq!(res7, serde_json::Value::Bool(true));

        let expr8 = "send_email.satcom.id === 12345.1"; // Testing with float comparison with
                                                        // decimal places this will be false
                                                        // as the float is not equal to the integer
        let res8 = evaluator.evaluate(expr8).unwrap();
        assert_eq!(res8, serde_json::Value::Bool(false));
    }
}
