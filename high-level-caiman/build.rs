use std::path::Path;

fn main() {
    let parser_config = lalrpop::Configuration::new();
    parser_config
        .process_file(Path::new("./src/parse/parser.lalrpop"))
        .unwrap();
}
