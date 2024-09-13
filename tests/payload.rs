#[cfg(test)]
mod tests {
    use exprimo::Evaluator;
    use serde_json::Value;
    use std::collections::HashMap;

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
            #[cfg(feature = "logging")]
            logger,
        );

        let expr1 = "send_email.status === \"success\"";

        let res1 = evaluator.evaluate(expr1).unwrap();

        assert_eq!(res1, Value::Bool(true));
    }
}
