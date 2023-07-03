use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "logging")]
use scribe_rust;

pub fn add_context(key: &str, json_str: &str, context: &mut HashMap<String, String>) {
    let json: Value = match serde_json::from_str(json_str) {
        Ok(json) => json,
        Err(_) => {
            context.insert(key.to_string(), json_str.to_string());
            return;
        }
    };
    let json_obj = json.as_object().unwrap();
    let context_obj = context.entry(key.to_string()).or_insert_with(String::new);
    let nested_obj = build_nested_object(json_obj);
    let nested_str = serde_json::to_string(&nested_obj).unwrap();
    context_obj.push_str(&format!(r#"{}"#, nested_str));
}

fn build_nested_object(json: &serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut nested_obj = serde_json::Map::new();
    for (key, value) in json.iter() {
        match value {
            Value::Null => {
                nested_obj.insert(key.clone(), Value::Null);
            }
            Value::Bool(b) => {
                nested_obj.insert(key.clone(), Value::Bool(*b));
            }
            Value::Number(n) => {
                nested_obj.insert(key.clone(), Value::Number(n.clone()));
            }
            Value::String(s) => {
                nested_obj.insert(key.clone(), Value::String(s.clone()));
            }
            Value::Array(arr) => {
                let nested_arr = arr
                    .iter()
                    .map(|v| match v {
                        Value::Null => Value::Null,
                        Value::Bool(b) => Value::Bool(*b),
                        Value::Number(n) => Value::Number(n.clone()),
                        Value::String(s) => Value::String(s.clone()),
                        Value::Array(_) => Value::Array(build_nested_array(v.as_array().unwrap())),
                        Value::Object(_) => {
                            Value::Object(build_nested_object(v.as_object().unwrap()))
                        }
                    })
                    .collect();
                nested_obj.insert(key.clone(), Value::Array(nested_arr));
            }
            Value::Object(obj) => {
                let nested_obj2 = build_nested_object(obj);
                nested_obj.insert(key.clone(), Value::Object(nested_obj2));
            }
        }
    }
    nested_obj
}

fn build_nested_array(json: &[Value]) -> Vec<Value> {
    json.iter()
        .map(|v| match v {
            Value::Null => Value::Null,
            Value::Bool(b) => Value::Bool(*b),
            Value::Number(n) => Value::Number(n.clone()),
            Value::String(s) => Value::String(s.clone()),
            Value::Array(arr) => Value::Array(build_nested_array(arr)),
            Value::Object(obj) => Value::Object(build_nested_object(obj)),
        })
        .collect()
}

pub fn to_json(context: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
    let mut json = HashMap::new();
    for (key, value) in context.iter() {
        json.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }
    json
}



fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "logging")]
    let logger = scribe_rust::Logger::default();

    let mut ctx = HashMap::new();

    add_context("send_email", r#"{"status": "success"}"#, &mut ctx);
    let engine = exprimo::Evaluator::new(
        to_json(&ctx),
        #[cfg(feature = "logging")]
        logger,
    );

    let expr = r#"send_email.status === "#;

    let result = engine.evaluate(expr)?;

    println!("send_email.status == 'success' => {}", result);

    Ok(())
}
