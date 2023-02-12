use std::path::PathBuf;

mod compile;
mod disasm;
mod yk;

use amd_dx_gsa::Atidxx64;
use clap::Parser;
use compile::compile_dxbc_to_rdna2;
use disasm::{disassemble_rdna2, print_output_depedencies};
use turnip_gfx_disasm::{ScalarAction, ScalarOutcome};
use yk::parse_gsfx;

use crate::{
    compile::compile_dxbc_to_amdil_text,
    disasm::{disassemble_amdil_text, resolve_amdil_text_dependencies},
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
    fxo_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let dll = unsafe { Atidxx64::try_load_lib_from(args.dll_path).expect("no library found") };

    let fxo = std::fs::read(args.fxo_path).expect("couldn't read fxo file");

    let (_, (gsvs, gsps)) = parse_gsfx(&fxo).expect("couldn't parse fxo file");

    if false {
        println!("Vertex Program");
        let vert_program = compile_dxbc_to_rdna2(&dll, gsvs.dxbc, disassemble_rdna2)
            .expect("couldn't compile vertex shader")
            .expect("couldn't disassemble vertex shader");
        print_output_depedencies(&vert_program);

        println!("\n\nFragment Program");
        let frag_program = compile_dxbc_to_rdna2(&dll, gsps.dxbc, disassemble_rdna2)
            .expect("couldn't compile frag shader")
            .expect("couldn't disassemble frag shader");
        print_output_depedencies(&frag_program);
    } else {
        println!("Vertex Program");
        let vert_program = compile_dxbc_to_amdil_text(&dll, gsvs.dxbc, disassemble_amdil_text)
            .expect("couldn't compile vertex shader")
            .expect("couldn't disassemble vertex shader");
        resolve_amdil_text_dependencies(vert_program);

        println!("\n\nFragment Program");
        let frag_program = compile_dxbc_to_amdil_text(&dll, gsps.dxbc, disassemble_amdil_text)
            .expect("couldn't compile frag shader")
            .expect("couldn't disassemble frag shader");
        resolve_amdil_text_dependencies(frag_program);
    }
}
