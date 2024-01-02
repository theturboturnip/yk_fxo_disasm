use std::collections::HashMap;
use std::fs::DirEntry;
use std::io::Write;
use std::path::PathBuf;

use amd_dx_gsa::Atidxx64;
use anyhow::anyhow;
use clap::Parser;
use yk_fxo_disasm::db::{ShaderDb, BytesType, ShaderStage, DisasmType, DbResult};
use yk_fxo_disasm::disasm::{print_output_depedencies, analyze_program};
use yk_fxo_disasm::yk::{parse_gsfx, parse_gsvs, parse_gsps};

use yk_fxo_disasm::{
    compile::compile_dxbc_to_amdil_text,
    disasm::disassemble_amdil_text,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// DLL path
    #[clap(long, value_parser, default_value = "assets/atidxx64.dll")]
    dll_path: PathBuf,

    /// Shader path
    #[clap(value_parser)]
    fxo_dir: PathBuf,

    #[clap(value_parser)]
    shader_category: String,

    #[clap(value_parser)]
    db_path: PathBuf,
}

fn path_to_shader_name(path: &PathBuf) -> String {
    path.file_name().unwrap().to_string_lossy().split_once(".").unwrap().0.to_string()
}

fn read_fxo(dll: &Atidxx64, category: &str, fxo_path: std::path::PathBuf, db: &mut ShaderDb) -> anyhow::Result<()> {
    let shader_name = &path_to_shader_name(&fxo_path);
    let fxo = std::fs::read(fxo_path)?;

    let (_, (gsvs, gsps)) = parse_gsfx(&fxo).map_err(|e| anyhow!("Failed to parse FXO {e:?}"))?;

    db.insert_bytes(category, shader_name, ShaderStage::Vertex, BytesType::DXBC, gsvs.dxbc)?;
    db.insert_bytes(category, shader_name, ShaderStage::Fragment, BytesType::DXBC, gsps.dxbc)?;
    
    compile_dxbc_to_amdil_text(dll, gsvs.dxbc, |text| {
        let disasm = std::str::from_utf8(text).unwrap();
        db.insert_disasm(category, shader_name, ShaderStage::Vertex, DisasmType::AMDIL, disasm)
    })??;

    compile_dxbc_to_amdil_text(dll, gsps.dxbc, |text| {
        let disasm = std::str::from_utf8(text).unwrap();
        db.insert_disasm(category, shader_name, ShaderStage::Fragment, DisasmType::AMDIL, disasm)
    })??;

    Ok(())
}

fn read_vso(dll: &Atidxx64, category: &str, vso_path: std::path::PathBuf, db: &mut ShaderDb) -> anyhow::Result<()> {
    let shader_name = &path_to_shader_name(&vso_path);
    let vso = std::fs::read(vso_path)?;

    let (_, gsvs) = parse_gsvs(&vso).map_err(|e| anyhow!("Failed to parse VSO {e:?}"))?;

    db.insert_bytes(category, shader_name, ShaderStage::Vertex, BytesType::DXBC, gsvs.dxbc)?;
    
    compile_dxbc_to_amdil_text(dll, gsvs.dxbc, |text| {
        let disasm = std::str::from_utf8(text).unwrap();
        db.insert_disasm(category, shader_name, ShaderStage::Vertex, DisasmType::AMDIL, disasm)
    })??;

    Ok(())
}

fn read_pso(dll: &Atidxx64, category: &str, pso_path: std::path::PathBuf, db: &mut ShaderDb) -> anyhow::Result<()> {
    let shader_name = &path_to_shader_name(&pso_path);
    let pso = std::fs::read(pso_path)?;

    let (_, gsps) = parse_gsps(&pso).map_err(|e| anyhow!("Failed to parse PSO {e:?}"))?;

    db.insert_bytes(category, shader_name, ShaderStage::Fragment, BytesType::DXBC, gsps.dxbc)?;
    
    compile_dxbc_to_amdil_text(dll, gsps.dxbc, |text| {
        let disasm = std::str::from_utf8(text).unwrap();
        db.insert_disasm(category, shader_name, ShaderStage::Fragment, DisasmType::AMDIL, disasm)
    })??;

    Ok(())
}

enum YkFileKind {
    FXO(PathBuf),
    VSO(PathBuf),
    PSO(PathBuf),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let dll = unsafe { Atidxx64::try_load_lib_from(args.dll_path).expect("no library found") };

    let mut db = ShaderDb::from_file(args.db_path)?;

    let shaders = std::fs::read_dir(args.fxo_dir)
        .expect("couldn't open directory")
        .filter_map(|file| {
            match file {
                Ok(file) => {
                    let name = file.file_name();
                    let name = name.to_str()?;
                    if name.ends_with(".fxo") {
                        Some(YkFileKind::FXO(file.path()))
                    } else if name.ends_with(".vso") {
                        Some(YkFileKind::VSO(file.path()))
                    } else if name.ends_with(".pso") {
                        Some(YkFileKind::PSO(file.path()))
                    } else {
                        None
                    }
                },
                Err(_) => None,
            }
        });

    for file in shaders {
        match file {
            YkFileKind::FXO(path) => read_fxo(&dll, &args.shader_category, path, &mut db)?,
            YkFileKind::VSO(path) => read_vso(&dll, &args.shader_category, path, &mut db)?,
            YkFileKind::PSO(path) => read_pso(&dll, &args.shader_category, path, &mut db)?,
        }
    }

    Ok(())

}
