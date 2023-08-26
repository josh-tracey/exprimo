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

    #[test]
    fn test_basic_evaluate_with_empty_strings() {
        let mut context = HashMap::new();

        context.insert(
            "a".to_string(),
            serde_json::Value::String("".to_string()),
        );
        context.insert("b".to_string(), serde_json::Value::Bool(true));

        #[cfg(feature = "logging")]
        let logger = Logger::default();

        let evaluator = Evaluator::new(
            context,
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
