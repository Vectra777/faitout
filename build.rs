#[cfg(all(windows, feature = "embed-icon"))]
fn main() {
    use std::path::Path;
    if !Path::new("assets/icon.ico").exists() {
        // Missing icon; skip embedding
        println!("cargo:warning=assets/icon.ico not found; skipping exe icon embedding");
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    if let Err(err) = res.compile() {
        println!("cargo:warning=winres compile failed: {}", err);
    }
}

#[cfg(not(all(windows, feature = "embed-icon")))]
fn main() {}
