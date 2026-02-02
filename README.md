stoml-args
==========

A lightweight CLI argument parser for Rust with seamless TOML configuration
support. Zero external dependencies beyond [`stoml`](https://github.com/parazyd/stoml)

Documentation on [docs.rs](https://docs.rs/stoml-args).

## Features

- **Layered Configuration**: CLI args → TOML config → Defaults
- **Config Loading**: Built-in `-c`/`--config` flag with pre-parsing
- **Type-safe**: Integer, Float, String, Boolean, Array, and Count types
- **Optional Args**: Support for truly optional arguments (no default, returns `None`)
- **Flexible Flags**: Short (`-v`), long (`--verbose`), combined (`-vvv`), with values (`-o file`, `-ofile`, `--output=file`)
- **Positional Arguments**: Required, optional, and variadic
- **Auto-generated Help**: `--help` and `--version` flags

## Quick Start

```rust
use stoml_args::{args, arg, ArgType};

fn main() {
    let arg_defs = vec![
        arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .default(8080i64)
            .toml_key("server.port"),
        arg("verbose")
            .short('v')
            .flag(),
    ];

    let matches = args("myapp")
        .version("1.0.0")
        .config_arg_default("config.toml")  // Auto -c/--config
        .arg(arg_defs[0].clone())
        .arg(arg_defs[1].clone())
        .parse()
        .unwrap_or_else(|e| e.exit())
        .with_defaults(&arg_defs);

    // TOML is automatically loaded and merged
    let port = matches.get_integer("port").unwrap();
    println!("Port: {}", port);
}
```

## Automatic Config File Loading

Config files are handled automatically with pre-parsing:

```rust
let matches = args("myapp")
    .config_arg()                        // Adds -c/--config flag
    // OR
    .config_arg_default("config.toml")   // Also tries config.toml if -c not given
    .arg(arg("port").toml_key("port"))
    .parse()?;

// TOML is loaded. CLI values override TOML values.
```

**How it works:**
1. Before full parsing, `-c`/`--config` is extracted
2. The TOML file is loaded
3. Full argument parsing happens
4. TOML values fill in any gaps (CLI takes precedence)

**Supported formats:**
```bash
myapp -c config.toml
myapp -cconfig.toml
myapp --config config.toml
myapp --config=config.toml
```

## Layered Configuration (Manual)

If you need more control, you can still load TOML manually:

```rust
let arg_defs = vec![
    arg("host")
        .long("host")
        .default("0.0.0.0")
        .toml_key("server.host"),
    arg("port")
        .short('p')
        .arg_type(ArgType::Integer)
        .default(8080i64)
        .toml_key("server.port"),
];

let matches = args("server")
    .arg(arg_defs[0].clone())
    .arg(arg_defs[1].clone())
    .parse()?
    .with_toml_file_optional("config.toml")?
    .with_defaults(&arg_defs);
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

Arguments are optional by default. If not provided and no default is set, getter methods return `None`:

```rust
let matches = args("myapp")
    .arg(arg("output").short('o').optional())  // explicit
    .parse()?;

// Returns None if not provided
if let Some(output) = matches.get_string("output") {
    println!("Output: {}", output);
}

// For optional booleans, use get_bool_opt()
match matches.get_bool_opt("debug") {
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

### Args Builder Methods

| Method | Description |
|--------|-------------|
| `new(name)` | Create new parser |
| `version(v)` | Set program version |
| `about(s)` | Set program description |
| `arg(a)` | Add an argument |
| `config_arg()` | Enable `-c`/`--config` flag |
| `config_arg_default(path)` | Enable config flag with default path |
| `disable_help()` | Disable auto `--help` |
| `disable_version()` | Disable auto `--version` |
| `parse()` | Parse from `std::env::args()` |
| `parse_from(args)` | Parse from custom args |

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

# Config file
-c config.toml      # short
--config=config.toml # long with equals

# Positional
myapp input.txt output.txt

# Stop parsing
myapp --flag -- -not-a-flag
```

## Error Handling

```rust
let matches = args("myapp")
    .arg(arg("input").required())
    .parse();

match matches {
    Ok(m) => { /* use matches */ }
    Err(e) if e.is_help() => {
        println!("{}", e);
        std::process::exit(0);
    }
    Err(e) if e.is_version() => {
        println!("{}", e);
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
