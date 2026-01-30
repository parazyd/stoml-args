stoml-args
==========

A lightweight CLI argument parser for Rust with seamless TOML configuration
support. Zero external dependencies beyond [`stoml`](https://github.com/parazyd/stoml)

## Features

- **Layered Configuration**: CLI args → TOML config → Defaults
- **Type-safe**: Integer, Float, String, Boolean, Array, and Count types
- **Optional Args**: Support for truly optional arguments (no default, returns `None`)
- **Flexible Flags**: Short (`-v`), long (`--verbose`), combined (`-vvv`), with values (`-o file`, `-ofile`, `--output=file`)
- **Positional Arguments**: Required, optional, and variadic
- **Auto-generated Help**: `--help` and `--version` flags
- **No serde**: Direct integration with `stoml` types

## Quick Start

```rust
use stoml_args::{args, arg, ArgType};

fn main() {
    let matches = args("myapp")
        .version("1.0.0")
        .about("My application")
        .arg(arg("config")
            .short('c')
            .long("config")
            .help("Config file path"))
        .arg(arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .default(8080i64)
            .help("Port number"))
        .arg(arg("verbose")
            .short('v')
            .long("verbose")
            .flag()
            .help("Enable verbose output"))
        .parse()
        .unwrap_or_else(|e| e.exit());

    let port = matches.get_integer("port").unwrap();
    let verbose = matches.get_bool("verbose");
    
    println!("Port: {}, Verbose: {}", port, verbose);
}
```

## Layered Configuration

```rust
use stoml_args::{args, arg, ArgType};

fn main() {
    // Define arguments with TOML key mappings
    let arg_defs = vec![
        arg("host")
            .long("host")
            .default("0.0.0.0")
            .toml_key("server.host"),  // Maps to [server] host = "..."
        arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .default(8080i64)
            .toml_key("server.port"),
    ];

    let matches = args("server")
        .arg(arg_defs[0].clone())
        .arg(arg_defs[1].clone())
        .parse()
        .unwrap_or_else(|e| e.exit())
        .with_toml_file_optional("config.toml")  // Load TOML (optional)
        .unwrap()
        .with_defaults(&arg_defs); // Apply defaults

    // CLI wins over TOML, TOML wins over defaults
    let host = matches.get_string("host").unwrap();
    let port = matches.get_integer("port").unwrap();
}
```

With `config.toml`:
```toml
[server]
host = "127.0.0.1"
port = 3000
```

Priority order (highest to lowest):
1. CLI arguments
2. TOML configuration
3. Defaults

## Argument Types

### Flags (Boolean)

```rust
arg("verbose")
    .short('v')
    .long("verbose")
    .flag()  // Sets type to Bool with default false
```

Supports `--no-verbose` to explicitly set false.

### Count

```rust
arg("verbosity")
    .short('v')
    .count()  // -v = 1, -vv = 2, -vvv = 3
```

### Values

```rust
// String (default)
arg("name").short('n').long("name")

// Integer
arg("port").arg_type(ArgType::Integer)

// Float
arg("rate").arg_type(ArgType::Float)

// Array (can be specified multiple times)
arg("include")
    .short('I')
    .arg_type(ArgType::Array)
// Usage: -I path1 -I path2
```

### Optional Arguments Without Defaults

Arguments are optional by default. If not provided and no default is set,
getter methods return `None`:

```rust
let matches = args("myapp")
    .arg(arg("config").short('c').optional())  // explicit optional
    .arg(arg("output").short('o'))              // implicit optional (default)
    .parse()
    .unwrap_or_else(|e| e.exit());

// Returns None if not provided
if let Some(config) = matches.get_string("config") {
    println!("Config: {}", config);
}

// For optional booleans, use get_bool_opt()
let debug = matches.get_bool_opt("debug");  // Option<bool>
match debug {
    Some(true) => println!("Debug enabled"),
    Some(false) => println!("Debug disabled"),
    None => println!("Debug not specified"),
}
```

### Positional Arguments

```rust
use stoml_args::pos;

args("cp")
    .arg(pos("source").required())
    .arg(pos("dest").required())
    .arg(pos("extras").variadic())  // Collects remaining args
```

## API Reference

### Arg Builder Methods

| Method | Description |
|--------|-------------|
| `short(c)` | Set short flag (e.g., `'v'` for `-v`) |
| `long(s)` | Set long flag (e.g., `"verbose"` for `--verbose`) |
| `arg_type(t)` | Set value type (`String`, `Integer`, `Float`, `Bool`, `Array`, `Count`) |
| `flag()` | Shorthand for boolean flag (default: false) |
| `count()` | Shorthand for count flag (default: 0) |
| `default(v)` | Set default value |
| `required()` | Mark as required |
| `optional()` | Mark as optional with no default (explicit) |
| `help(s)` | Set help description |
| `toml_key(s)` | Set TOML key path (e.g., `"server.port"`) |
| `value_name(s)` | Set placeholder in help (e.g., `"FILE"`) |
| `variadic()` | Accept multiple values (positional only) |

### Matches Methods

| Method | Description |
|--------|-------------|
| `get(name)` | Get `Option<&Value>` |
| `get_string(name)` | Get `Option<&str>` |
| `get_integer(name)` | Get `Option<i64>` |
| `get_float(name)` | Get `Option<f64>` |
| `get_bool(name)` | Get `bool` (defaults to `false`) |
| `get_bool_opt(name)` | Get `Option<bool>` (truly optional) |
| `get_array(name)` | Get `Option<&Array>` |
| `get_count(name)` | Get `i64` (defaults to `0`) |
| `get_count_opt(name)` | Get `Option<i64>` (truly optional) |
| `contains(name)` | Check if argument was provided |
| `with_toml(table)` | Merge TOML table (CLI takes precedence) |
| `with_toml_file(path)` | Load and merge TOML file |
| `with_toml_file_optional(path)` | Load TOML file if it exists |
| `with_defaults(args)` | Apply default values |
| `remaining()` | Get args after `--` |
| `to_table()` | Convert to `stoml::Table` |

## Flag Formats Supported

```bash
# Boolean flags
-v                  # short
--verbose           # long
--no-verbose        # negation

# Value flags
-o file             # short with space
-ofile              # short attached
--output file       # long with space
--output=file       # long with equals

# Combined short flags
-abc                # same as -a -b -c
-vvv                # count: 3

# Positional
myapp input.txt output.txt

# Stop parsing
myapp --flag -- -not-a-flag
```

## Error Handling

```rust
let matches = args("myapp")
    .arg(arg("config").required())
    .parse();

match matches {
    Ok(m) => { /* use matches */ }
    Err(e) if e.is_help() => {
        println!("{}", e);  // Print help
        std::process::exit(0);
    }
    Err(e) if e.is_version() => {
        println!("{}", e);  // Print version
        std::process::exit(0);
    }
    Err(e) => {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

// Or simply:
let matches = args("myapp").parse().unwrap_or_else(|e| e.exit());
```

## License

GPL-3.0
