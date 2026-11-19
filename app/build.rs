use std::path::Path;

/// Cat photos placed in `renderer/cat/` as happy|cheer|worried|angry|slapped.(png|jpg|webp|gif|avif)
/// are embedded with the UI; the list is baked in at build time (the Electron version scanned at run time).
fn main() {
    let dir = Path::new("../renderer/cat");
    println!("cargo:rerun-if-changed=../renderer/cat");
    let mut found: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let file = e.file_name().to_string_lossy().into_owned();
            let lower = file.to_lowercase();
            if let Some((stem, ext)) = lower.rsplit_once('.')
                && ["happy", "cheer", "worried", "angry", "slapped"].contains(&stem)
                && ["png", "jpg", "jpeg", "webp", "gif", "avif"].contains(&ext)
            {
                found.push((stem.to_string(), format!("cat/{file}")));
            }
        }
    }
    found.sort();
    let json = format!(
        "{{{}}}",
        found
            .iter()
            .map(|(k, v)| format!("\"{k}\":\"{v}\""))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("cargo:rustc-env=DL_CAT_IMAGES={json}");
    tauri_build::build();
}
