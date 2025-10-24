# Exprimo

<img src="assets/logo.png" width="250" title="rusty">

Exprimo is a **reliable** and **JavaScript-compliant** expression evaluator written in Rust, designed for rule-based engines and dynamic expression evaluation. Inspired by angular-expressions, it's built to be simple, fast, and production-ready.

## Description

Exprimo parses and evaluates JavaScript expressions efficiently and securely. It utilizes the power of Rust and its excellent memory safety guarantees to provide a reliable and fast JavaScript expression evaluator with proper error handling and JavaScript semantics compliance.

**Perfect for:**
- Rule-based engines
- Dynamic configuration
- Conditional logic evaluation
- Template expressions
- Business rule processing

## Features

✅ **JavaScript-Compliant** - Follows JavaScript semantics for intuitive expression writing  
✅ **Robust Error Handling** - Gracefully handles edge cases (division by zero, NaN, Infinity)  
✅ **Type Coercion** - Supports both loose (`==`) and strict (`===`) equality with proper type coercion  
✅ **Rich Type Support** - Numbers, strings, booleans, arrays, objects, null, NaN, Infinity  
✅ **Custom Functions** - Extend with your own Rust functions  
✅ **Built-in Methods** - Array and object methods (`.length`, `.includes()`, `.hasOwnProperty()`)  
✅ **String Escapes** - Proper handling of escape sequences (`\n`, `\t`, `\\`, etc.)  
✅ **Production-Ready** - Comprehensive test coverage (43+ tests)

## Installation

Add Exprimo to your `Cargo.toml`:

```toml
[dependencies]
exprimo = "*"
```

Or install via cargo:

```bash
cargo add exprimo
```

### Optional Logging

Enable trace logging for debugging:

```toml
[dependencies]
exprimo = { version = "*", features = ["logging"] }
```

This will install [Scribe Rust](https://github.com/josh-tracey/scribe-rust). Set `LOG_LEVEL=TRACE` in environment variables for logs to output.

## Quick Start

```rust
use exprimo::Evaluator;
use serde_json::Value;
use std::collections::HashMap;

// Set up context with variables
let mut context = HashMap::new();
context.insert("user_age".to_string(), Value::Number(30.into()));
context.insert("user_status".to_string(), Value::String("active".to_string()));
context.insert("user_premium".to_string(), Value::Bool(true));

// Create evaluator
let evaluator = Evaluator::new(context, HashMap::new());

// Evaluate expressions
let result = evaluator.evaluate("user_age >= 18 && user_status === 'active'").unwrap();
assert_eq!(result, Value::Bool(true));

// Ternary operator
let result = evaluator.evaluate("user_premium ? 'VIP' : 'Standard'").unwrap();
assert_eq!(result, Value::String("VIP".to_string()));

// Type coercion with ==
let result = evaluator.evaluate("user_premium == 1").unwrap();
assert_eq!(result, Value::Bool(true)); // true == 1 with type coercion
```

## Supported Operations

### Arithmetic Operators

```javascript
a + b    // Addition (also string concatenation)
a - b    // Subtraction
a * b    // Multiplication
a / b    // Division (returns Infinity for division by zero)
a % b    // Modulo
-a       // Unary negation
+a       // Unary plus
```

### Comparison Operators

```javascript
a == b   // Loose equality (with type coercion)
a != b   // Loose inequality
a === b  // Strict equality (no type coercion)
a !== b  // Strict inequality
a > b    // Greater than
a < b    // Less than
a >= b   // Greater than or equal
a <= b   // Less than or equal
```

### Logical Operators

```javascript
a && b   // Logical AND
a || b   // Logical OR
!a       // Logical NOT
```

### Ternary Operator

```javascript
condition ? valueIfTrue : valueIfFalse
```

### Grouping

```javascript
(a + b) * c
```

## Type Coercion

Exprimo implements JavaScript-compliant type coercion for the `==` operator:

```rust
// String to number coercion
evaluator.evaluate("'5' == 5").unwrap();        // true
evaluator.evaluate("'5' === 5").unwrap();       // false (strict)

// Boolean to number coercion
evaluator.evaluate("true == 1").unwrap();       // true
evaluator.evaluate("false == 0").unwrap();      // true

// Empty string to number
evaluator.evaluate("'' == 0").unwrap();         // true
```

## Truthiness Rules

Exprimo follows JavaScript truthiness semantics:

| Value | Truthy/Falsy | Example |
|-------|--------------|---------|
| `true` | Truthy | `true ? 'yes' : 'no'` → `'yes'` |
| `false` | Falsy | `false ? 'yes' : 'no'` → `'no'` |
| `null` | Falsy | `null ? 'yes' : 'no'` → `'no'` |
| `0` | Falsy | `0 ? 'yes' : 'no'` → `'no'` |
| `NaN` | Falsy | `NaN ? 'yes' : 'no'` → `'no'` |
| `""` (empty string) | Falsy | `"" ? 'yes' : 'no'` → `'no'` |
| Non-zero numbers | Truthy | `42 ? 'yes' : 'no'` → `'yes'` |
| Non-empty strings | Truthy | `"hello" ? 'yes' : 'no'` → `'yes'` |
| **All arrays** | **Truthy** | `[] ? 'yes' : 'no'` → `'yes'` |
| **All objects** | **Truthy** | `{} ? 'yes' : 'no'` → `'yes'` |

**Note:** Arrays and objects are **always truthy**, even if empty (JavaScript standard behavior).

## Special Values

### Infinity and NaN

Exprimo properly handles `Infinity` and `NaN`:

```rust
// Division by zero returns Infinity
evaluator.evaluate("5 / 0").unwrap();     // Infinity (no error!)
evaluator.evaluate("-5 / 0").unwrap();    // -Infinity

// Invalid conversions return NaN
evaluator.evaluate("'abc' * 2").unwrap(); // NaN (no error!)

// NaN comparisons
evaluator.evaluate("NaN == NaN").unwrap();  // false (JavaScript behavior)
evaluator.evaluate("NaN === NaN").unwrap(); // false

// Infinity identifier
evaluator.evaluate("Infinity > 1000000").unwrap(); // true
```

**Note:** Due to `serde_json::Number` limitations, `NaN` and `Infinity` are represented as best-effort approximations. For production use with heavy `NaN`/`Infinity` usage, consider a custom `Value` type.

### Undefined

The `undefined` identifier returns `null` (closest JSON equivalent):

```rust
evaluator.evaluate("undefined").unwrap(); // null
```

## String Escape Sequences

Exprimo processes common escape sequences:

```rust
evaluator.evaluate("'line1\\nline2'").unwrap();      // "line1\nline2"
evaluator.evaluate("'col1\\tcol2'").unwrap();        // "col1\tcol2"
evaluator.evaluate("'quote: \\'hello\\''").unwrap(); // "quote: 'hello'"
evaluator.evaluate("\"quote: \\\"hello\\\"\"").unwrap(); // "quote: \"hello\""
evaluator.evaluate("'path\\\\to\\\\file'").unwrap(); // "path\to\file"
```

**Supported escapes:** `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`

## Type Conversions

### String to Number

```rust
evaluator.evaluate("'42' * 2").unwrap();      // 84
evaluator.evaluate("'abc' * 2").unwrap();     // NaN
evaluator.evaluate("'' * 2").unwrap();        // 0 (empty string → 0)
evaluator.evaluate("'Infinity' > 100").unwrap(); // true
```

### Array to Number

```rust
evaluator.evaluate("[] * 5").unwrap();        // 0 (empty array → 0)
evaluator.evaluate("[42] * 2").unwrap();      // 84 (single element)
evaluator.evaluate("[1, 2] * 2").unwrap();    // NaN (multiple elements)
```

### Object to Number

```rust
evaluator.evaluate("{} * 2").unwrap();        // NaN (objects → NaN)
```

## Built-in Properties and Methods

### Arrays

Arrays are represented by `serde_json::Value::Array`.

#### `.length`

Returns the number of elements in an array.

```rust
let mut context = HashMap::new();
context.insert("myArray".to_string(), Value::Array(vec![
    Value::Number(10.into()),
    Value::Number(20.into()),
    Value::Number(30.into()),
]));
let evaluator = Evaluator::new(context, HashMap::new());

evaluator.evaluate("myArray.length").unwrap(); // 3
```

#### `.includes(valueToFind)`

Checks if an array contains `valueToFind` using SameValueZero comparison (strict equality where `NaN` equals `NaN`).

```rust
evaluator.evaluate("[1, 'foo', null].includes('foo')").unwrap(); // true
evaluator.evaluate("[1, 2, 3].includes(4)").unwrap();            // false
evaluator.evaluate("[NaN].includes(NaN)").unwrap();              // true (special case)
```

### Objects

Objects are represented by `serde_json::Value::Object`.

#### `.hasOwnProperty(key)`

Checks if an object contains the specified `key` as its own direct property.

```rust
let mut context = HashMap::new();
let mut obj = serde_json::Map::new();
obj.insert("name".to_string(), Value::String("Alice".to_string()));
obj.insert("age".to_string(), Value::Number(30.into()));
context.insert("myObject".to_string(), Value::Object(obj));
let evaluator = Evaluator::new(context, HashMap::new());

evaluator.evaluate("myObject.hasOwnProperty('name')").unwrap();   // true
evaluator.evaluate("myObject.hasOwnProperty('gender')").unwrap(); // false
evaluator.evaluate("myObject.hasOwnProperty(123)").unwrap();      // false (coerced to "123")
```

## Custom Functions

Extend Exprimo with your own Rust functions by implementing the `CustomFunction` trait.

### Example: `toUpperCase` Function

```rust
use exprimo::{Evaluator, CustomFunction, CustomFuncError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
struct ToUpperCase;

impl CustomFunction for ToUpperCase {
    fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError> {
        if args.len() != 1 {
            return Err(CustomFuncError::ArityError { expected: 1, got: args.len() });
        }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err(CustomFuncError::ArgumentError("Argument must be a string".to_string())),
        }
    }
}

fn main() {
    let context = HashMap::new();
    let mut custom_functions: HashMap<String, Arc<dyn CustomFunction>> = HashMap::new();
    custom_functions.insert("toUpperCase".to_string(), Arc::new(ToUpperCase));
    
    let evaluator = Evaluator::new(context, custom_functions);
    
    let result = evaluator.evaluate("toUpperCase('hello world')").unwrap();
    assert_eq!(result, Value::String("HELLO WORLD".to_string()));
}
```

### Custom Function Requirements

- Implement `exprimo::CustomFunction` trait (also requires `std::fmt::Debug`)
- Implement the `call` method with signature: `fn call(&self, args: &[Value]) -> Result<Value, CustomFuncError>`
- Handle argument validation (count and types)
- Return `Ok(Value)` on success or `Err(CustomFuncError)` on failure
- Wrap in `Arc::new()` before inserting into the custom functions map

## Real-World Example: Rule Engine

```rust
use exprimo::Evaluator;
use serde_json::Value;
use std::collections::HashMap;

// Rule engine context
let mut context = HashMap::new();
context.insert("user_age".to_string(), Value::Number(25.into()));
context.insert("user_country".to_string(), Value::String("US".to_string()));
context.insert("user_score".to_string(), Value::Number(850.into()));
context.insert("user_verified".to_string(), Value::Bool(true));
context.insert("user_tags".to_string(), Value::Array(vec![
    Value::String("premium".to_string()),
    Value::String("verified".to_string()),
]));

let evaluator = Evaluator::new(context, HashMap::new());

// Rule 1: Eligibility check
let eligible = evaluator.evaluate(
    "user_age >= 18 && user_verified && user_score >= 700"
).unwrap();
assert_eq!(eligible, Value::Bool(true));

// Rule 2: Discount calculation
let discount = evaluator.evaluate(
    "user_tags.includes('premium') ? 20 : 10"
).unwrap();
assert_eq!(discount, Value::Number(20.into()));

// Rule 3: Complex condition
let approved = evaluator.evaluate(
    "(user_country === 'US' || user_country === 'CA') && user_score > 800"
).unwrap();
assert_eq!(approved, Value::Bool(true));
```

## Error Handling

Exprimo provides detailed error types:

```rust
use exprimo::EvaluationError;

let result = evaluator.evaluate("unknown_variable");
match result {
    Ok(value) => println!("Result: {}", value),
    Err(EvaluationError::Node(e)) => println!("Node error: {}", e),
    Err(EvaluationError::TypeError(e)) => println!("Type error: {}", e),
    Err(EvaluationError::CustomFunction(e)) => println!("Function error: {}", e),
}
```

## Known Limitations

1. **serde_json::Number Constraints**
   - `NaN` and `Infinity` don't serialize perfectly to JSON
   - Workarounds are in place, but consider a custom `Value` type for production

2. **Complex Literals**
   - Only empty array `[]` and empty object `{}` literals are supported
   - Complex literals like `[1, 2, 3]` or `{a: 1, b: 2}` are not yet implemented
   - **Workaround:** Pass complex structures via context

3. **Object Literal Ambiguity**
   - `{}` in expression context is parsed as a block statement (JavaScript quirk)
   - **Workaround:** Use variables or wrap in parentheses (future support)

## Testing

Run the test suite:

```bash
cargo test
```

Run with logging:

```bash
LOG_LEVEL=TRACE cargo test --features "logging"
```

Run examples:

```bash
cargo run --example basic
LOG_LEVEL=TRACE cargo run --features "logging" --example basic
```

## Performance

Exprimo is designed for performance:
- Minimal allocations
- Efficient AST traversal
- Zero-copy where possible
- Rust's memory safety without runtime overhead

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

### Recent Improvements (Latest Version)

- ✅ Division by zero returns `Infinity`/`NaN` instead of errors
- ✅ Invalid type conversions return `NaN` instead of errors
- ✅ Proper NaN comparison semantics (`NaN != NaN`)
- ✅ Arrays and objects are now truthy (JavaScript standard)
- ✅ Type coercion for `==` operator
- ✅ Separate strict equality `===` without coercion
- ✅ `Infinity`, `NaN`, `undefined` identifiers
- ✅ String escape sequence processing
- ✅ Array to number conversion
- ✅ SameValueZero for `Array.includes()`

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

Exprimo is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.

## Credits

Inspired by [angular-expressions](https://github.com/peerigon/angular-expressions).

Built with ❤️ using Rust and [rslint_parser](https://github.com/rslint/rslint).

