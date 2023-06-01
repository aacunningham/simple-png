use anyhow::Context;
use simple_png::PNG;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    let output_dir = Path::new("benchmark");
    let test_images = fs::read_dir("tests/png-suite/")
        .context("Failed to read png-suite folder")?
        .filter_map(|entry| entry.ok())
        .filter(|p| {
            let path = p.path();
            path.is_file()
                && path.extension() == Some(OsStr::new("png"))
                && !path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map(|file_name| file_name.starts_with('x'))
                    .unwrap_or(true)
        });
    let mut processed_images = Vec::with_capacity(test_images.size_hint().1.unwrap_or(50));

    for image in test_images {
        let image_path = image.path();
        let test_name = image_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap();
        let orig_name = PathBuf::from(format!("{test_name}-orig.png"));
        let spng_name = PathBuf::from(format!("{test_name}-spng.png"));
        fs::copy(image_path.clone(), output_dir.join(orig_name.clone())).context(format!(
            "Failed to copy from {} to {}",
            image_path.to_str().unwrap(),
            orig_name.to_str().unwrap(),
        ))?;
        fs::write(
            output_dir.join(spng_name),
            PNG::decode(&fs::read(image_path.clone())?)
                .context(format!(
                    "Failed to decode {}.",
                    image_path.to_str().unwrap()
                ))?
                .encode(),
        )?;
        processed_images.push(test_name.to_owned());
    }
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)?;
    let results = serde_json::json!({
        "date": now,
        "processed_images": processed_images,
    });
    fs::write(output_dir.join("test_results.json"), results.to_string())?;
    Ok(())
}
