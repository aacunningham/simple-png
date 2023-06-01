use simple_png::PNG;

fn main() -> anyhow::Result<()> {
    let file_name = std::env::args().nth(1).unwrap();
    let input = std::fs::read(file_name)?;
    let output = PNG::decode(&input)?.encode();
    std::fs::write("output.png", output)?;
    Ok(())
}
