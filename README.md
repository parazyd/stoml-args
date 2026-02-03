stoml-args
==========

A lightweight CLI argument parser for Rust with seamless TOML configuration
support. Zero external dependencies beyond [`stoml`](https://github.com/parazyd/stoml).

[Documentation](https://docs.rs/stoml-args)

## Features

- **Layered Configuration**: CLI args → TOML config → Defaults
- **Config Loading**: Built-in `-c`/`--config` flag
- **Config Templates**: Auto-create default config if missing
- **Type-safe**: Integer, Float, String, Boolean, Array, and Count types
- **Optional Args**: Support for truly optional arguments (no default, returns `None`)
- **Flexible Flags**: Short (`-v`), long (`--verbose`), combined (`-vvv`), with values
- **Positional Arguments**: Required, optional, and variadic
- **Auto-generated Help**: `--help` and `--version` flags

## Quick Start

```rust
use stoml_args::{args, arg, ArgType};

const DEFAULT_CONFIG: &str = r#"
[server]
port = 8080
host = "0.0.0.0"
"#;

fn main() {
    let arg_defs = vec![
        arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .default(8080i64)
            .toml_key("server.port"),
        arg("host")
            .long("host")
            .default("0.0.0.0")
            .toml_key("server.host"),
    ];

    let matches = args("myapp")
        .version("1.0.0")
        .config_arg_default("config.toml")  // -c/--config, default path
        .config_template(DEFAULT_CONFIG)    // Create if missing
        .arg(arg_defs[0].clone())
        .arg(arg_defs[1].clone())
        .parse()
        .unwrap_or_else(|e| e.exit())
        .with_defaults(&arg_defs);

    // Config is auto-created if missing, then loaded
    // CLI args override TOML values
    println!("Port: {}", matches.get_integer("port").unwrap());
}
```

## Automatic Config File Handling

The parser handles config files automatically:

```rust
args("myapp")
    .config_arg()                        // Just adds -c/--config flag
    // OR
    .config_arg_default("config.toml")   // Also sets default path
    .config_template(TOML_CONTENT)       // Write this if file missing
    .config_required(true)               // Error if no config exists
    .parse()?;
```

**Behavior:**

| Scenario | `config_template` set | `config_required` | Result |
|----------|----------------------|-------------------|--------|
| File exists | - | - | Load it |
| File missing | Yes | - | Create from template, load it |
| File missing | No | false (default) | Continue without config |
| File missing | No | true | Error: MissingConfig |

**Supported formats:**
```bash
myapp -c config.toml
myapp -cconfig.toml
myapp --config config.toml
myapp --config=config.toml
```

## Layered Configuration

Priority order (highest to lowest):
1. CLI arguments
2. TOML configuration
3. Defaults

```rust
// config.toml has: port = 3000
// User runs: myapp -p 8080

matches.get_integer("port")  // Returns 8080 (CLI wins)
```

## Argument Types

### Flags (Boolean)

```rust
arg("verbose")
    .short('v')
    .long("verbose")
    .flag()  // Bool with default false
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

// Array (can be repeated)
arg("include")
    .short('I')
    .arg_type(ArgType::Array)
// Usage: -I path1 -I path2
```

### Optional Arguments Without Defaults

```rust
let matches = args("myapp")
    .arg(arg("output").short('o').optional())
    .parse()?;

// Returns None if not provided
if let Some(output) = matches.get_string("output") {
    println!("Output: {}", output);
}

// For optional booleans
match matches.get_bool_opt("debug") {
    Some(true) => println!("Debug on"),
    Some(false) => println!("Debug off"),
    None => println!("Debug not specified"),
}
```

### Positional Arguments

```rust
use stoml_args::pos;

args("cp")
    .arg(pos("source").required())
    .arg(pos("dest").required())
    .arg(pos("extras").variadic())
```

## API Reference

### Args Builder Methods

| Method | Description |
|--------|-------------|
| `new(name)` | Create new parser |
| `version(v)` | Set program version |
| `about(s)` | Set program description |
| `arg(a)` | Add an argument |
| `config_arg()` | Enable `-c`/`--config` flag |
| `config_arg_default(path)` | Enable config flag with default path |
| `config_template(content)` | TOML to write if config missing |
| `config_required(bool)` | Error if no config (default: false) |
| `disable_help()` | Disable auto `--help` |
| `disable_version()` | Disable auto `--version` |
| `parse()` | Parse from `std::env::args()` |
| `parse_from(args)` | Parse from custom args |

### Arg Builder Methods

| Method | Description |
|--------|-------------|
| `short(c)` | Short flag (`'v'` for `-v`) |
| `long(s)` | Long flag (`"verbose"` for `--verbose`) |
| `arg_type(t)` | Value type (`String`, `Integer`, `Float`, `Bool`, `Array`, `Count`) |
| `flag()` | Boolean flag (default: false) |
| `count()` | Count flag (default: 0) |
| `default(v)` | Default value |
| `required()` | Mark as required |
| `optional()` | Mark as optional (explicit) |
| `help(s)` | Help description |
| `toml_key(s)` | TOML key path (`"server.port"`) |
| `value_name(s)` | Help placeholder (`"FILE"`) |
| `variadic()` | Accept multiple values (positional only) |

### Matches Methods

| Method | Description |
|--------|-------------|
| `get(name)` | `Option<&Value>` |
| `get_string(name)` | `Option<&str>` |
| `get_integer(name)` | `Option<i64>` |
| `get_float(name)` | `Option<f64>` |
| `get_bool(name)` | `bool` (default: false) |
| `get_bool_opt(name)` | `Option<bool>` |
| `get_array(name)` | `Option<&Array>` |
| `get_count(name)` | `i64` (default: 0) |
| `get_count_opt(name)` | `Option<i64>` |
| `contains(name)` | Check if provided |
| `with_toml(table)` | Merge TOML table |
| `with_toml_file(path)` | Load and merge TOML |
| `with_toml_file_optional(path)` | Load if exists |
| `with_defaults(args)` | Apply defaults |
| `remaining()` | Args after `--` |
| `to_table()` | Convert to `stoml::Table` |

## Error Handling

```rust
let matches = args("myapp")
    .arg(arg("input").required())
    .parse();

match matches {
    Ok(m) => { /* use m */ }
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
