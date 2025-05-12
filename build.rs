// build.rs

fn main() {
    // Nur für Windows-Ziele ausführen
    if std::env::var("TARGET").unwrap().contains("windows") {
        // Compile the Windows manifest by referencing the .rc file
        embed_resource::compile("qrlan.rc", embed_resource::NONE);
    }
}
