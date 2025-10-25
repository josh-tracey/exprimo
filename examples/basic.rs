use std::collections::HashMap;
use std::error::Error;

use exprimo;

fn main() -> Result<(), Box<dyn Error>> {
    let mut ctx = HashMap::new();
    ctx.insert("x".to_string(), serde_json::Value::Number(5.into()));
    let engine = exprimo::Evaluator::new(
        ctx,
        HashMap::new(), // custom_functions
    );

    let result = engine.evaluate("x == 5")?;

    println!("x = {}", result);

    Ok(())
}
