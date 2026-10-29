use wasmtime::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::default();

    // Read file
    let data = std::fs::read("wasi-plugins/test_plugin.wasm")?;
    println!("Read {} bytes", data.len());
    println!(
        "First 32 bytes: {:?}",
        &data[..std::cmp::min(32, data.len())]
    );

    // Try to parse as module
    let module = Module::new(&engine, &data)?;
    println!("Module loaded!");

    Ok(())
}
