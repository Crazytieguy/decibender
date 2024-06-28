fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../.env");
    for item in dotenvy::dotenv_iter()? {
        let (key, value) = item?;
        println!("cargo::rustc-env={key}={value}");
    }
    tauri_build::build();
    Ok(())
}
