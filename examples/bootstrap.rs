use stoml_args::{arg, args, ArgType};

const DEFAULT_CONFIG: &str = r#"# MyApp Configuration
# This file was auto-generated with default values.
# Edit as needed.

[server]
host = "127.0.0.1"
port = 8080
workers = 4

[logging]
level = "info"
file = "app.log"
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define arguments with TOML mappings
    let arg_defs = vec![
        arg("host")
            .short('H')
            .long("host")
            .help("Server bind address")
            .toml_key("server.host"),
        arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .help("Server port")
            .toml_key("server.port"),
        arg("workers")
            .short('w')
            .long("workers")
            .arg_type(ArgType::Integer)
            .help("Number of worker threads")
            .toml_key("server.workers"),
        arg("log-level")
            .short('l')
            .long("log-level")
            .help("Logging level")
            .toml_key("logging.level"),
    ];

    // Build parser with:
    // - config_arg_default: sets the default config path
    // - config_template: content to write if file doesn't exist
    // - config_required: whether to error if no config (after template creation)
    let parser = args("bootstrap-example")
        .version("1.0.0")
        .about("Demonstrates automatic config file creation")
        .config_arg_default("config.toml") // Default config path
        .config_template(DEFAULT_CONFIG) // Write this if file missing
        .config_required(false); // Don't error if missing (template will create it anyway)

    let parser = arg_defs.iter().fold(parser, |p, a| p.arg(a.clone()));

    // Parse - if config.toml doesn't exist, it's created from template automatically!
    let matches = parser
        .parse()
        .unwrap_or_else(|e| e.exit())
        .with_defaults(&arg_defs);

    // Use the values - CLI overrides TOML overrides defaults
    println!("Configuration:");
    if let Some(cfg) = matches.get_string("config") {
        println!("  Config file: {}", cfg);
    }
    println!(
        "  Host: {}",
        matches.get_string("host").unwrap_or("not set")
    );
    println!("  Port: {}", matches.get_integer("port").unwrap_or(0));
    println!("  Workers: {}", matches.get_integer("workers").unwrap_or(0));
    println!(
        "  Log level: {}",
        matches.get_string("log-level").unwrap_or("not set")
    );

    Ok(())
}
