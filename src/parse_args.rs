use std::{collections::HashMap, process::exit};

pub fn parse_args() -> HashMap<String, String> {
    let (args, argv) = argmap::new()
        .booleans(&["h", "help", "f", "file"])
        .parse(std::env::args());

    if argv.contains_key("h") || argv.contains_key("help") {
        indoc::printdoc![
            r#"usage: {} {{OPTIONS}} [FILE]

      Count the number of bytes, words, or lines in a file or stdin.

        -f, --file  file to output final dir
        -h, --help    Show this message.
    "#,
            args.get(0).unwrap_or(&"???".to_string())
        ];

        exit(0)
    }

    let mut args_map: HashMap<String, String> = HashMap::new();

    let stdin_file = "-".to_string();

    let file = argv
        .get("file")
        .and_then(|v| v.first()) // --file=file
        .or_else(|| argv.get("f").and_then(|v| v.first())) // -f file
        .or_else(|| args.get(1)) // first positional arg after $0
        .unwrap_or(&stdin_file) // default value: "-"
        .as_str();

    args_map.insert("file".to_string(), file.to_string());

    return args_map;
}
