use syntect::dumps::dump_to_file;
use syntect::parsing::SyntaxSet;

fn main() {
    let ss_defaults = SyntaxSet::load_defaults_newlines();
    dump_to_file(&ss_defaults, "assets/syntax.packdump").unwrap();
    println!("Dumped syntax set to assets/syntax.packdump");
}
