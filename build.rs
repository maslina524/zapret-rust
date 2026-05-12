use std::fs;
use std::path::{Path, PathBuf};
use std::env;

fn main() {
    let folders_to_copy = ["bin", "configs", "lists"];
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .unwrap();
    
    println!("cargo:warning=Copying resources to {}", target_dir.display());
    
    for folder in folders_to_copy {
        let source_dir = PathBuf::from(folder);
        let dest_dir = target_dir.join(folder);
        
        if source_dir.exists() {
            println!("cargo:warning=Copying {} -> {}", folder, dest_dir.display());
            
            if dest_dir.exists() {
                fs::remove_dir_all(&dest_dir).unwrap_or(());
            }
            
            if let Err(e) = copy_dir_all(&source_dir, &dest_dir) {
                println!("cargo:warning=Copy Error {}: {}", folder, e);
            } else {
                println!("cargo:warning={} is copied!", folder);
            }
            
            // Указываем Cargo пересобирать при изменениях
            println!("cargo:rerun-if-changed={}", folder);
        } else {
            println!("cargo:warning=The `{}` folder was not found", folder);
        }
    }
    
    println!("cargo:warning=Resources copied to {}", target_dir.display());
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}