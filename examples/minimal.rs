use stoml_args::{ArgType, arg, args, pos};

fn main() {
    // Define arguments
    let arg_defs = vec![
        arg("output")
            .short('o')
            .long("output")
            .default("out.txt")
            .toml_key("output")
            .help("Output file"),
        arg("verbose")
            .short('v')
            .long("verbose")
            .flag()
            .toml_key("verbose")
            .help("Enable verbose output"),
        arg("count")
            .short('n')
            .long("count")
            .arg_type(ArgType::Integer)
            .default(10i64)
            .toml_key("count")
            .help("Number of iterations"),
    ];

    // Parse with automatic config file support
    let matches = args("minimal")
        .version("0.1.0")
        .about("A minimal example")
        .config_arg_default("config.toml")
        .arg(arg_defs[0].clone())
        .arg(arg_defs[1].clone())
        .arg(arg_defs[2].clone())
        .arg(pos("input").required().help("Input file"))
        .parse()
        .unwrap_or_else(|e| e.exit())
        .with_defaults(&arg_defs);

    let input = matches.get_string("input").unwrap();
    let output = matches.get_string("output").unwrap();
    let verbose = matches.get_bool("verbose");
    let count = matches.get_integer("count").unwrap();

    println!("Input:   {}", input);
    println!("Output:  {}", output);
    println!("Verbose: {}", verbose);
    println!("Count:   {}", count);
}
