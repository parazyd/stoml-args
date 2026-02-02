mod error;
mod parser;
use parser::ArgParser;

pub use error::{Error, Result};
pub use stoml::{Array, Table, Value};

use std::collections::HashMap;
use std::env;
use std::path::Path;

/// The type of value an argument accepts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// A string value
    String,
    /// An integer value
    Integer,
    /// A floating-point value
    Float,
    /// A boolean flag (presence = true, --no-flag = false)
    Bool,
    /// Multiple values (can be specified multiple times)
    Array,
    /// A count (each occurrence increments, e.g., -vvv = 3)
    Count,
}

/// Definition of a single argument
#[derive(Debug, Clone)]
pub struct Arg {
    /// The primary name (used as key for lookup and TOML matching)
    pub name: String,
    /// Short flag (e.g., 'v' for -v)
    pub short: Option<char>,
    /// Long flag (e.g., "verbose" for --verbose)
    pub long: Option<String>,
    /// The type of value this argument accepts
    pub arg_type: ArgType,
    /// Default value if not provided
    pub default: Option<Value>,
    /// Whether this argument is required
    pub required: bool,
    /// Help description
    pub help: Option<String>,
    /// The TOML key path to look up (defaults to name)
    /// Use dots for nested keys: "server.port"
    pub toml_key: Option<String>,
    /// Value name shown in help (e.g., "FILE" in "--config <FILE>")
    pub value_name: Option<String>,
    /// Whether this is a positional argument
    pub positional: bool,
    /// Position index for positional arguments
    pub position: Option<usize>,
    /// Whether this positional can accept multiple values (must be last)
    pub variadic: bool,
}

impl Arg {
    /// Create a new argument with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short: None,
            long: None,
            arg_type: ArgType::String,
            default: None,
            required: false,
            help: None,
            toml_key: None,
            value_name: None,
            positional: false,
            position: None,
            variadic: false,
        }
    }

    /// Create a new positional argument
    pub fn positional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short: None,
            long: None,
            arg_type: ArgType::String,
            default: None,
            required: false,
            help: None,
            toml_key: None,
            value_name: None,
            positional: true,
            position: None,
            variadic: false,
        }
    }

    /// Set the short flag (e.g., 'v' for -v)
    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Set the long flag (e.g., "verbose" for --verbose)
    pub fn long(mut self, s: impl Into<String>) -> Self {
        self.long = Some(s.into());
        self
    }

    /// Set the argument type
    pub fn arg_type(mut self, t: ArgType) -> Self {
        self.arg_type = t;
        self
    }

    /// Convenience method for boolean flags
    pub fn flag(mut self) -> Self {
        self.arg_type = ArgType::Bool;
        self.default = Some(Value::Boolean(false));
        self
    }

    /// Convenience method for count flags (-vvv = 3)
    pub fn count(mut self) -> Self {
        self.arg_type = ArgType::Count;
        self.default = Some(Value::Integer(0));
        self
    }

    /// Set the default value
    pub fn default(mut self, v: impl Into<Value>) -> Self {
        self.default = Some(v.into());
        self
    }

    /// Mark this argument as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Explicitly mark this argument as optional (this is the default)
    ///
    /// Use this for clarity when you want an optional argument with no default.
    /// When not provided, `get_string()`, `get_integer()`, etc. will return `None`.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self.default = None;
        self
    }

    /// Set the help description
    pub fn help(mut self, s: impl Into<String>) -> Self {
        self.help = Some(s.into());
        self
    }

    /// Set the TOML key path (use dots for nesting: "server.port")
    pub fn toml_key(mut self, s: impl Into<String>) -> Self {
        self.toml_key = Some(s.into());
        self
    }

    /// Set the value name shown in help
    pub fn value_name(mut self, s: impl Into<String>) -> Self {
        self.value_name = Some(s.into());
        self
    }

    /// Mark this positional as variadic (accepts multiple values, must be last)
    pub fn variadic(mut self) -> Self {
        self.variadic = true;
        self.arg_type = ArgType::Array;
        self
    }
}

/// Builder for creating an argument parser
#[derive(Debug, Clone)]
pub struct Args {
    /// Program name
    name: String,
    /// Program version
    version: Option<String>,
    /// Program description
    about: Option<String>,
    /// Defined arguments
    args: Vec<Arg>,
    /// Positional argument count
    positional_count: usize,
    /// Whether to auto-add help flag
    auto_help: bool,
    /// Whether to auto-add version flag
    auto_version: bool,
    /// Whether to auto-add config file flag (-c/--config)
    auto_config: bool,
    /// Default config file path (used if -c/--config not provided)
    default_config: Option<String>,
}

impl Args {
    /// Create a new argument parser with the given program name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            about: None,
            args: Vec::new(),
            positional_count: 0,
            auto_help: true,
            auto_version: true,
            auto_config: false,
            default_config: None,
        }
    }

    /// Set the program version
    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.version = Some(v.into());
        self
    }

    /// Set the program description
    pub fn about(mut self, s: impl Into<String>) -> Self {
        self.about = Some(s.into());
        self
    }

    /// Add an argument
    pub fn arg(mut self, mut arg: Arg) -> Self {
        if arg.positional {
            arg.position = Some(self.positional_count);
            self.positional_count += 1;
        }
        self.args.push(arg);
        self
    }

    /// Disable automatic help flag
    pub fn disable_help(mut self) -> Self {
        self.auto_help = false;
        self
    }

    /// Disable automatic version flag
    pub fn disable_version(mut self) -> Self {
        self.auto_version = false;
        self
    }

    /// Enable automatic config file flag (-c/--config)
    ///
    /// This adds a `-c`/`--config` argument that is parsed first, before other arguments.
    /// If provided, the TOML file is loaded and merged automatically.
    ///
    /// # Example
    /// ```ignore
    /// let matches = args("myapp")
    ///     .config_arg()
    ///     .arg(arg("port").default(8080i64).toml_key("server.port"))
    ///     .parse()?;
    /// // If user runs: myapp -c config.toml
    /// // The TOML is already loaded and merged
    /// ```
    pub fn config_arg(mut self) -> Self {
        self.auto_config = true;
        self
    }

    /// Enable automatic config file flag with a default path
    ///
    /// Like `config_arg()`, but also tries to load from the default path
    /// if `-c`/`--config` is not provided.
    ///
    /// # Example
    /// ```ignore
    /// let matches = args("myapp")
    ///     .config_arg_default("config.toml")
    ///     .parse()?;
    /// // Loads config.toml if it exists, even without -c flag
    /// ```
    pub fn config_arg_default(mut self, path: impl Into<String>) -> Self {
        self.auto_config = true;
        self.default_config = Some(path.into());
        self
    }

    /// Parse arguments from the command line
    pub fn parse(self) -> Result<Matches> {
        self.parse_from(env::args().skip(1).collect())
    }

    /// Parse arguments from a given iterator
    pub fn parse_from(mut self, args: Vec<String>) -> Result<Matches> {
        // Pre-scan for config file if auto_config is enabled
        let config_table = if self.auto_config {
            let config_path = self.extract_config_path(&args);
            self.load_config_file(config_path.as_deref())?
        } else {
            None
        };

        // Add auto flags
        if self.auto_config {
            self.args.push(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("Path to configuration file")
                    .value_name("FILE"),
            );
        }
        if self.auto_help {
            self.args.push(
                Arg::new("help")
                    .short('h')
                    .long("help")
                    .flag()
                    .help("Print help information"),
            );
        }
        if self.auto_version && self.version.is_some() {
            self.args.push(
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .flag()
                    .help("Print version information"),
            );
        }

        let parser = ArgParser::new(&self.args);
        let mut matches = parser.parse(args)?;

        // Check for help/version
        if self.auto_help && matches.get_bool("help") {
            return Err(Error::Help(self.format_help()));
        }
        if self.auto_version && matches.get_bool("version") {
            return Err(Error::Version(self.format_version()));
        }

        // Merge TOML config (CLI values take precedence since they're already in matches)
        if let Some(table) = config_table {
            matches.merge_toml(&table, "");
        }

        // Check for missing required arguments (after help/version and TOML merge)
        for arg in &self.args {
            if arg.required && !matches.values.contains_key(&arg.name) {
                if arg.positional {
                    return Err(Error::MissingPositional {
                        name: arg.name.clone(),
                        position: arg.position.unwrap_or(0),
                    });
                } else {
                    return Err(Error::MissingRequired {
                        name: arg.name.clone(),
                    });
                }
            }
        }

        // Store metadata
        matches.program_name = self.name;

        Ok(matches)
    }

    /// Extract config path from args without full parsing
    fn extract_config_path(&self, args: &[String]) -> Option<String> {
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            // --config=path or --config path
            if let Some(rest) = arg.strip_prefix("--config") {
                if let Some(path) = rest.strip_prefix('=') {
                    return Some(path.to_string());
                } else if rest.is_empty() {
                    return iter.next().cloned();
                }
            }
            // -c path or -cpath
            if let Some(rest) = arg.strip_prefix("-c") {
                if rest.is_empty() {
                    return iter.next().cloned();
                } else {
                    return Some(rest.to_string());
                }
            }
            // Handle combined flags like -vc path (config is last)
            if arg.starts_with('-') && !arg.starts_with("--") && arg.contains('c') {
                let chars: Vec<char> = arg[1..].chars().collect();
                if let Some(pos) = chars.iter().position(|&ch| ch == 'c') {
                    // If 'c' is the last char, next arg is the value
                    if pos == chars.len() - 1 {
                        return iter.next().cloned();
                    }
                    // Otherwise, rest after 'c' is the value
                    let rest: String = chars[pos + 1..].iter().collect();
                    if !rest.is_empty() {
                        return Some(rest);
                    }
                }
            }
        }
        // Fall back to default config path
        self.default_config.clone()
    }

    /// Load config file, returns None if file doesn't exist (when using default)
    fn load_config_file(&self, path: Option<&str>) -> Result<Option<Table>> {
        match path {
            Some(p) => {
                // Explicit path provided - error if not found
                if self.default_config.as_deref() == Some(p) && !std::path::Path::new(p).exists() {
                    // Default config doesn't exist - that's OK
                    Ok(None)
                } else {
                    // Explicit -c/--config or default exists - load it
                    Ok(Some(stoml::parse_file(p)?))
                }
            }
            None => Ok(None),
        }
    }

    /// Format help message
    fn format_help(&self) -> String {
        let mut help = String::new();

        // Usage line
        help.push_str(&format!("Usage: {}", self.name));

        // Collect positionals
        let positionals: Vec<_> = self.args.iter().filter(|a| a.positional).collect();

        if self.args.iter().any(|a| !a.positional) {
            help.push_str(" [OPTIONS]");
        }

        for arg in &positionals {
            let name = arg.value_name.as_deref().unwrap_or(&arg.name);
            if arg.required {
                help.push_str(&format!(" <{}>", name.to_uppercase()));
            } else {
                help.push_str(&format!(" [{}]", name.to_uppercase()));
            }
            if arg.variadic {
                help.push_str("...");
            }
        }

        help.push('\n');

        // Description
        if let Some(about) = &self.about {
            help.push('\n');
            help.push_str(about);
            help.push('\n');
        }

        // Positional arguments
        if !positionals.is_empty() {
            help.push_str("\nArguments:\n");
            for arg in &positionals {
                let name = arg.value_name.as_deref().unwrap_or(&arg.name);
                help.push_str(&format!("  <{}>", name.to_uppercase()));
                if let Some(h) = &arg.help {
                    help.push_str(&format!("  {}", h));
                }
                help.push('\n');
            }
        }

        // Options
        let options: Vec<_> = self.args.iter().filter(|a| !a.positional).collect();
        if !options.is_empty() {
            help.push_str("\nOptions:\n");
            for arg in &options {
                let mut line = String::from("  ");

                // Short flag
                if let Some(c) = arg.short {
                    line.push_str(&format!("-{}", c));
                    if arg.long.is_some() {
                        line.push_str(", ");
                    }
                } else {
                    line.push_str("    ");
                }

                // Long flag
                if let Some(l) = &arg.long {
                    line.push_str(&format!("--{}", l));
                }

                // Value placeholder
                if arg.arg_type != ArgType::Bool && arg.arg_type != ArgType::Count {
                    let vname = arg
                        .value_name
                        .as_deref()
                        .unwrap_or(&arg.name)
                        .to_uppercase();
                    line.push_str(&format!(" <{}>", vname));
                }

                // Pad for alignment
                let pad = 28usize.saturating_sub(line.len());
                line.push_str(&" ".repeat(pad));

                // Help text
                if let Some(h) = &arg.help {
                    line.push_str(h);
                }

                // Default value
                if let Some(d) = &arg.default
                    && !matches!(d, Value::Boolean(false) | Value::Integer(0))
                {
                    line.push_str(&format!(" [default: {}]", d));
                }

                help.push_str(&line);
                help.push('\n');
            }
        }

        help
    }

    /// Format version message
    fn format_version(&self) -> String {
        format!(
            "{} {}",
            self.name,
            self.version.as_deref().unwrap_or("unknown")
        )
    }
}

/// The result of parsing arguments
#[derive(Debug, Clone)]
pub struct Matches {
    /// Parsed values
    values: HashMap<String, Value>,
    /// Program name
    program_name: String,
    /// Raw remaining arguments
    remaining: Vec<String>,
}

impl Matches {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
            program_name: String::new(),
            remaining: Vec::new(),
        }
    }

    /// Merge with TOML configuration (TOML values are used only if not already set)
    pub fn with_toml(mut self, table: &Table) -> Self {
        self.merge_toml(table, "");
        self
    }

    /// Merge with TOML file (reads and parses the file)
    pub fn with_toml_file<P: AsRef<Path>>(self, path: P) -> Result<Self> {
        let table = stoml::parse_file(path)?;
        Ok(self.with_toml(&table))
    }

    /// Merge with TOML file if it exists (does not error if missing)
    pub fn with_toml_file_optional<P: AsRef<Path>>(self, path: P) -> Result<Self> {
        if path.as_ref().exists() {
            self.with_toml_file(path)
        } else {
            Ok(self)
        }
    }

    /// Apply defaults from argument definitions
    pub fn with_defaults(mut self, args: &[Arg]) -> Self {
        for arg in args {
            if !self.values.contains_key(&arg.name)
                && let Some(default) = &arg.default
            {
                self.values.insert(arg.name.clone(), default.clone());
            }
        }
        self
    }

    fn merge_toml(&mut self, table: &Table, prefix: &str) {
        for (key, value) in table.iter() {
            let full_key = if prefix.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", prefix, key)
            };

            // Recursively handle nested tables
            if let Some(inner) = value.as_table() {
                self.merge_toml(inner, &full_key);
            }

            // Only insert if not already set (CLI takes precedence)
            self.values.entry(full_key).or_insert_with(|| value.clone());
        }
    }

    /// Check if an argument was provided
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Get a value by name
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    /// Get a string value
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.values.get(name).and_then(|v| v.as_str())
    }

    /// Get a string value or default
    pub fn get_string_or(&self, name: &str, default: &str) -> String {
        self.get_string(name)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }

    /// Get an integer value
    pub fn get_integer(&self, name: &str) -> Option<i64> {
        self.values.get(name).and_then(|v| v.as_integer())
    }

    /// Get an integer value or default
    pub fn get_integer_or(&self, name: &str, default: i64) -> i64 {
        self.get_integer(name).unwrap_or(default)
    }

    /// Get a float value
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.values.get(name).and_then(|v| v.as_float())
    }

    /// Get a float value or default
    pub fn get_float_or(&self, name: &str, default: f64) -> f64 {
        self.get_float(name).unwrap_or(default)
    }

    /// Get a boolean value (returns false if not present)
    pub fn get_bool(&self, name: &str) -> bool {
        self.values
            .get(name)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get an optional boolean value (returns None if not present)
    pub fn get_bool_opt(&self, name: &str) -> Option<bool> {
        self.values.get(name).and_then(|v| v.as_bool())
    }

    /// Get an array value
    pub fn get_array(&self, name: &str) -> Option<&Array> {
        self.values.get(name).and_then(|v| v.as_array())
    }

    /// Get a count value (returns 0 if not present)
    pub fn get_count(&self, name: &str) -> i64 {
        self.values
            .get(name)
            .and_then(|v| v.as_integer())
            .unwrap_or(0)
    }

    /// Get an optional count value (returns None if not present)
    pub fn get_count_opt(&self, name: &str) -> Option<i64> {
        self.values.get(name).and_then(|v| v.as_integer())
    }

    /// Get remaining unparsed arguments
    pub fn remaining(&self) -> &[String] {
        &self.remaining
    }

    /// Get all values as a reference to the internal map
    pub fn values(&self) -> &HashMap<String, Value> {
        &self.values
    }

    /// Convert to a Table (useful for serialization or further processing)
    pub fn to_table(&self) -> Table {
        let mut table = Table::new();
        for (key, value) in &self.values {
            // Handle dotted keys by creating nested tables
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() == 1 {
                table.insert(key.clone(), value.clone());
            } else {
                // Navigate/create nested structure
                let mut current = &mut table;
                for (i, part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 {
                        current.insert((*part).to_string(), value.clone());
                    } else {
                        if !current.contains_key(part) {
                            current.insert((*part).to_string(), Value::Table(Table::new()));
                        }
                        if let Some(Value::Table(t)) = current.get_mut(part) {
                            current = t;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        table
    }
}

/// Convenience function to create a new Args builder
#[inline]
pub fn args(name: impl Into<String>) -> Args {
    Args::new(name)
}

/// Convenience function to create a new Arg
#[inline]
pub fn arg(name: impl Into<String>) -> Arg {
    Arg::new(name)
}

/// Convenience function to create a positional Arg
#[inline]
pub fn pos(name: impl Into<String>) -> Arg {
    Arg::positional(name)
}
