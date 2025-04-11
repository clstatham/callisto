use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The file to execute
    #[clap(value_parser)]
    file: String,
}

fn main() -> anyhow::Result<()> {
    let Args { file } = Args::try_parse()?;
    let input = std::fs::read_to_string(file)?;
    let result = callisto_interpreter::vm::execute_str(&input)?;
    println!("{:?}", result);
    Ok(())
}
