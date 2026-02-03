use stoml_args::{arg, args, ArgType};

const DEFAULT_CONFIG: &str = r#"# Server Configuration
# Auto-generated default config. Edit as needed.

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[logging]
file = "server.log"

[tls]
enabled = false
# cert = "/path/to/cert.pem"
# key = "/path/to/key.pem"

# features = ["metrics", "tracing"]
"#;

fn main() {
    // Define all arguments with their full configuration
    let arg_defs = vec![
        // Server configuration
        arg("host")
            .short('H')
            .long("host")
            .help("Bind address")
            .default("0.0.0.0")
            .toml_key("server.host")
            .value_name("ADDR"),
        arg("port")
            .short('p')
            .long("port")
            .arg_type(ArgType::Integer)
            .help("Port to listen on")
            .default(8080i64)
            .toml_key("server.port")
            .value_name("PORT"),
        arg("workers")
            .short('w')
            .long("workers")
            .arg_type(ArgType::Integer)
            .help("Number of worker threads")
            .default(4i64)
            .toml_key("server.workers"),
        // Logging
        arg("verbose")
            .short('v')
            .long("verbose")
            .count()
            .help("Increase verbosity (can be repeated: -vvv)"),
        arg("quiet")
            .short('q')
            .long("quiet")
            .flag()
            .help("Suppress all output"),
        arg("log-file")
            .short('l')
            .long("log-file")
            .help("Log to file instead of stderr")
            .toml_key("logging.file")
            .value_name("PATH"),
        // Features
        arg("feature")
            .short('f')
            .long("feature")
            .arg_type(ArgType::Array)
            .help("Enable feature (can be repeated)")
            .toml_key("features")
            .value_name("NAME"),
        // TLS
        arg("tls")
            .long("tls")
            .flag()
            .help("Enable TLS")
            .toml_key("tls.enabled"),
        arg("cert")
            .long("cert")
            .help("TLS certificate path")
            .toml_key("tls.cert")
            .value_name("PATH"),
        arg("key")
            .long("key")
            .help("TLS private key path")
            .toml_key("tls.key")
            .value_name("PATH"),
    ];

    // Build the parser with automatic config handling
    let parser = args("myserver")
        .version("1.0.0")
        .about("A demonstration web server with layered configuration")
        .config_arg_default("config.toml") // -c/--config, defaults to config.toml
        .config_template(DEFAULT_CONFIG); // Create with this content if missing

    // Add all arguments
    let parser = arg_defs.iter().fold(parser, |p, a| p.arg(a.clone()));

    // Parse - config is auto-created if missing!
    let matches = match parser.parse() {
        Ok(m) => m,
        Err(e) => e.exit(),
    };

    // Apply defaults for any remaining unset values
    let matches = matches.with_defaults(&arg_defs);

    // Now use the values
    let host = matches.get_string("host").unwrap_or("0.0.0.0");
    let port = matches.get_integer("port").unwrap_or(8080);
    let workers = matches.get_integer("workers").unwrap_or(4);
    let verbosity = matches.get_count("verbose");
    let quiet = matches.get_bool("quiet");
    let log_file = matches.get_string("log-file");
    let tls_enabled = matches.get_bool("tls");

    // Print configuration
    if !quiet {
        println!("Server Configuration:");
        if let Some(config) = matches.get_string("config") {
            println!("  Config file: {}", config);
        }
        println!("  Host: {}", host);
        println!("  Port: {}", port);
        println!("  Workers: {}", workers);
        println!("  Verbosity: {}", verbosity);

        if let Some(f) = log_file {
            println!("  Log file: {}", f);
        }

        if let Some(features) = matches.get_array("feature") {
            print!("  Features: ");
            for (i, f) in features.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                if let Some(s) = f.as_str() {
                    print!("{}", s);
                }
            }
            println!();
        }

        if tls_enabled {
            println!("  TLS: enabled");
            if let Some(cert) = matches.get_string("cert") {
                println!("    Certificate: {}", cert);
            }
            if let Some(key) = matches.get_string("key") {
                println!("    Private key: {}", key);
            }
        }
    }
}
