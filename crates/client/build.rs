use glsl_to_spirv::ShaderType;
use std::error::Error;
use std::fs;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let base_path = "/home/brynn/dev/octane";
    let resources_path = format!("{}/{}/", base_path, "resources");
    let assets_path = format!("{}/{}/", base_path, "assets");

    println!("cargo:rerun-if-changed={}*", resources_path);

    for entry in fs::read_dir(resources_path)? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path = entry.path();

            let shader_type =
                in_path
                    .extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vs" => Some(ShaderType::Vertex),
                        "fs" => Some(ShaderType::Fragment),
                        _ => None,
                    });

            if let None = shader_type {
                continue;
            }

            let source = fs::read_to_string(&in_path)?;

            let mut compilation = glsl_to_spirv::compile(&source, shader_type.unwrap())?;

            let mut compiled_bytes = vec![];

            compilation.read_to_end(&mut compiled_bytes)?;

            let out_path = format!(
                "{}/{}.spirv",
                assets_path,
                in_path.file_name().unwrap().to_string_lossy(),
            );

            if fs::metadata(&assets_path).is_err() {
                fs::create_dir("/home/brynn/dev/octane/assets/")?;
            }

            if fs::metadata(&out_path).is_ok() {
                println!("hello");
                fs::remove_file(&out_path)?;
            }

            fs::write(&out_path, &compiled_bytes)?;
        }
    }

    Ok(())
}
