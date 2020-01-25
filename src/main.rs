#[macro_use]
extern crate clap;
#[macro_use]
extern crate dotenv_codegen;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let verbosity = matches.occurrences_of("verbose");
    match verbosity {
        0 => (),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => println!("Don't be silly"),
    }

    println!("{}", dotenv!("VERSION"));
    println!("{}", std::env::var("PWD").unwrap_or_default());
    println!("{}", std::env::var("HOME").unwrap_or_default());

    let v = vec![1, 2, 3];

    // v[99];
    std::process::exit(-1);
}
