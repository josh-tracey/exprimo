use exprimo::Evaluator;
use std::collections::HashMap;

#[cfg(feature = "logging")]
use scribe_rust::Logger;

#[cfg(test)]
#[test]
fn test_primitives() {
    let context = HashMap::new();

    #[cfg(feature = "logging")]
    let logger = Logger::default();

    let evaluator = Evaluator::new(
        context,
        #[cfg(feature = "logging")]
        logger,
    );

    let expr1 = "null == undefined";
    let res1 = evaluator.evaluate(&expr1).unwrap();

    assert_eq!(res1, true);
}
