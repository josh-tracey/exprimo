# Censura

Censura is a JavaScript evaluator written in Rust, inspired by the functionality of 
angular-expression-js. Designed to be simple and blazingly fast.

## Description

Censura parses and evaluates JavaScript expressions efficiently and securely. 
It utilizes the power of Rust and its excellent memory safety guarantees to provide a reliable
and fast JavaScript expression evaluator.

## Installation

Before you can use Censura, you need to have Rust installed on your system. 
If you don't have it installed, you can download Rust from the official website 
[here](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can install Censura by running:

```bash
cargo install censura
```

## Usage

First, you need to import Censura and create an instance of `Evaluator`:

```rust
use censura::Evaluator;
use std::collections::HashMap;

let mut context = HashMap::new();
context.insert("key", "value");

let evaluator = Evaluator::new(context);
```

Then, you can evaluate JavaScript expressions:

```rust
let result = evaluator.evaluate("key == 'value'").unwrap();
println!("The result is {}", result);
```

## Examples

Please see the `examples` directory in the GitHub repository for more usage examples.

## Contributing

Contributions to Censura are welcome! Please submit a pull request on GitHub.

## License

Censura is licensed under the MIT license. Please see the `LICENSE` file in the GitHub 
repository for more information.
