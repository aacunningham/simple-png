use simple_png::PNG;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let verbosity = if args.first().unwrap() == "-v" {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Error
    };
    pretty_env_logger::formatted_builder()
        .filter_level(verbosity)
        .init();
    let file_name = args.last().unwrap();
    let input = std::fs::read(file_name)?;
    let output = PNG::decode(&input)?.encode();
    std::fs::write("output.png", output)?;
    Ok(())
}
