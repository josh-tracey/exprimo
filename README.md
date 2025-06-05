# Exprimo

<img src="assets/logo.png" width="250" title="rusty">

**CAUTION:** Beware of Sneaky Bugs in the Early Project Stages!
There will be bug, probably alot at first :D

Exprimo is a JavaScript evaluator written in Rust, inspired by the functionality of 
angular-expressions. Designed to be simple and blazingly fast.

## Description

Exprimo parses and evaluates JavaScript expressions efficiently and securely. 
It utilizes the power of Rust and its excellent memory safety guarantees to provide a reliable
and fast JavaScript expression evaluator.

## Installation

Before you can use Exprimo, you need to have Rust installed on your system. 
If you don't have it installed, you can download Rust from the official website 
[here](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can install Exprimo by running:

```bash
cargo add exprimo
```

Trace logging to console can be added to the package, it is by default disabled as probably 
don't need it unless working on it, or need to debug AST error if required.

This will install [Scribe Rust](https://github.com/josh-tracey/scribe-rust) and will need LOG_LEVEL=TRACE in environment variables for logs to output.

```toml
exprimo = { version = "*", features = ["logging"]
```

## Usage

First, you need to import Exprimo and create an instance of `Evaluator`:

```rust
use exprimo::{Evaluator, CustomFunction}; // CustomFunction for example context
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc; // For custom functions map

// 1. Set up your context (variables accessible in expressions)
let mut context = HashMap::new();
context.insert("user_name".to_string(), Value::String("Alice".to_string()));
context.insert("user_age".to_string(), Value::Number(30.into()));

// 2. Define custom functions map (even if empty)
let custom_functions: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
// Example: To use custom functions, you would populate this map:
//   #[derive(Debug)] struct MyToUpper;
//   impl CustomFunction for MyToUpper { /* ... */ }
//   custom_functions.insert("toUpperCase".to_string(), Arc::new(MyToUpper));

// 3. Create the evaluator instance
// (Assuming no logger for this basic example for brevity; add logger if feature "logging" is enabled)
let evaluator = Evaluator::new(context, custom_functions);
```

Then, you can evaluate JavaScript expressions:

```rust
let age_expr = "user_age > 25";
// Example with a (hypothetical) custom function if it were registered:
// let name_expr = "toUpperCase(user_name)";

let is_older = evaluator.evaluate(age_expr).unwrap();
// let uppercased_name = evaluator.evaluate(name_expr).unwrap();


println!("Is older: {}", is_older); // >Is older: true
// println!("Uppercased name: {}", uppercased_name);
```

## Truthiness Rules

Exprimo evaluates the truthiness of values as follows:

- **Booleans**: `true` is truthy, `false` is falsy.
- **Null**: `null` is falsy.
- **Numbers**: `0` and `NaN` are falsy. All other numbers (including negative numbers and Infinity) are truthy.
- **Strings**: Empty strings (`""`) are falsy. All other strings are truthy.
- **Arrays**: Empty arrays (`[]`) are currently treated as falsy. This behavior might differ from standard JavaScript, where empty arrays are truthy.
- **Objects**: Empty objects (`{}`) are currently treated as falsy. This behavior might differ from standard JavaScript, where empty objects are truthy.

## Custom Functions

You can extend Exprimo's capabilities by defining your own functions in Rust and making them available to the evaluator.

Custom functions must implement the `exprimo::CustomFunction` trait. This trait requires a `call` method that takes a slice of `serde_json::Value` arguments (`&[Value]`) and returns a `Result<Value, exprimo::CustomFuncError>`.

**Example: A `toUpperCase` function**

```rust
use exprimo::{Evaluator, CustomFunction, CustomFuncError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt; // Required for #[derive(Debug)] on the custom function struct

#[derive(Debug)] // The CustomFunction trait requires the Debug trait.
struct ToUpperCase;

impl CustomFunction for ToUpperCase {
    fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError> {
        if args.len() != 1 {
            // Return an arity error if the wrong number of arguments are provided.
            return Err(CustomFuncError::ArityError { expected: 1, got: args.len() });
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            // Return an argument error if the type is not as expected.
            _ => Err(CustomFuncError::ArgumentError("Argument must be a string".to_string())),
        }
    }
}

fn main() { // Example main, in real use integrate into your app
    // Context can be empty if not needed
    let context = HashMap::new();

    // Register your custom function
    let mut custom_functions: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_functions.insert("toUpperCase".to_string(), Arc::new(ToUpperCase));

    // Create the evaluator with the custom functions
    // (Assuming no logger for simplicity in this example)
    let evaluator = Evaluator::new(context, custom_functions);

    // Evaluate an expression using the custom function
    let result = evaluator.evaluate("toUpperCase('hello world')").unwrap();
    assert_eq!(result, Value::String("HELLO WORLD".to_string()));
    println!("toUpperCase('hello world') => {}", result); // >toUpperCase('hello world') => "HELLO WORLD"
}
```

Key points for custom functions:
- Your struct must implement `exprimo::CustomFunction` (which also requires `std::fmt::Debug`).
- The `call` method receives arguments as `&[Value]`. You are responsible for:
    - Checking argument count (arity).
    - Checking argument types.
    - Performing the function logic.
    - Returning `Ok(Value)` on success or `Err(exprimo::CustomFuncError)` on failure (e.g., `CustomFuncError::ArityError`, `CustomFuncError::ArgumentError`).
- Wrap your function instance in `Arc::new()` before inserting into the `custom_functions` map.
- The keys in the `custom_functions` map are the names used to call the functions in expressions.

## Built-in Properties and Methods

Exprimo provides a few built-in properties and methods for common operations, primarily for arrays and objects.

### Arrays

Arrays in Exprimo are represented by `serde_json::Value::Array`.

-   **`.length`**: Returns the number of elements in an array.
    *   Example: If `myArray` is `[10, 20, 30]`:
        ```javascript
        myArray.length // Evaluates to 3
        ```

-   **`.includes(valueToFind)`**: Checks if an array contains `valueToFind`.
    *   It uses an abstract equality comparison similar to JavaScript's `SameValueZero` (e.g., `NaN` is equal to `NaN`, `+0` is equal to `-0`).
    *   Returns `true` if the value is found, `false` otherwise.
    *   Example:
        ```javascript
        [1, "foo", null].includes("foo") // Evaluates to true
        [1, 2, 3].includes(4)           // Evaluates to false
        ```

### Objects

Objects in Exprimo are represented by `serde_json::Value::Object`.

-   **`.hasOwnProperty(key)`**: Checks if an object contains the specified `key` as its own direct property.
    *   The `key` argument is coerced to a string. For example, if you pass a number `123`, it will be treated as the string `"123"`.
    *   Returns `true` if the key is found, `false` otherwise.
    *   Example: If `myObject` is `{ "name": "Alice", "age": 30 }`:
        ```javascript
        myObject.hasOwnProperty('name')    // Evaluates to true
        myObject.hasOwnProperty('gender')  // Evaluates to false
        ({ "123": "value" }).hasOwnProperty(123) // Evaluates to true (123 is coerced to "123")
        ```

## Examples

Running examples

```bash
LOG_LEVEL=TRACE cargo run --features "logging" --example basic
```

## Contributing

Contributions to Exprimo are welcome! Please submit a pull request on GitHub.

## License

Exprimo is licensed under the MIT license. Please see the `LICENSE` file in the GitHub 
repository for more information.
