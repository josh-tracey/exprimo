use exprimo::Evaluator;
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn test_primitives() {
    let context = HashMap::new();

    let evaluator = Evaluator::new(
        context,
        HashMap::new(), // custom_functions
    );

    let expr1 = "1/2";
    let res1 = evaluator.evaluate(&expr1).unwrap();

    assert_eq!(res1, 0.5);
}
