# Exprimo

Exprimo is a JavaScript evaluator written in Rust, inspired by the functionality of 
angular-expression-js. Designed to be simple and blazingly fast.

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
cargo install exprimo
```

## Usage

First, you need to import Exprimo and create an instance of `Evaluator`:

```rust
use exprimo::Evaluator;
use std::collections::HashMap;

let mut context = HashMap::new();
context.insert("a".to_string(), serde_json::Value::Bool(true));        
context.insert("b".to_string(), serde_json::Value::Bool(false));

let evaluator = Evaluator::new(context);
```

Then, you can evaluate JavaScript expressions:

```rust
let result = evaluator.evaluate("a && b").unwrap();
println!("The result is {}", result);

//--outputs
// >The result is false
```

## Examples

Please see the `examples` directory in the GitHub repository for more usage examples.

## Contributing

Contributions to Exprimo are welcome! Please submit a pull request on GitHub.

## License

Exprimo is licensed under the MIT license. Please see the `LICENSE` file in the GitHub 
repository for more information.
