use std::path::PathBuf;

mod compile;
mod disasm;
mod yk;
mod db;

use amd_dx_gsa::Atidxx64;
use clap::Parser;
use disasm::print_output_depedencies;
use yk::parse_gsfx;

use crate::{
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
    fxo_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let dll = unsafe { Atidxx64::try_load_lib_from(args.dll_path).expect("no library found") };

    let fxo = std::fs::read(args.fxo_path).expect("couldn't read fxo file");

    let (_, (gsvs, gsps)) = parse_gsfx(&fxo).expect("couldn't parse fxo file");

    // if false {
    //     println!("Vertex Program");
    //     let vert_program = compile_dxbc_to_rdna2(&dll, gsvs.dxbc, disassemble_rdna2)
    //         .expect("couldn't compile vertex shader")
    //         .expect("couldn't disassemble vertex shader");
    //     print_output_depedencies(&vert_program);

    //     println!("\n\nFragment Program");
    //     let frag_program = compile_dxbc_to_rdna2(&dll, gsps.dxbc, disassemble_rdna2)
    //         .expect("couldn't compile frag shader")
    //         .expect("couldn't disassemble frag shader");
    //     print_output_depedencies(&frag_program);
    // } else {
        println!("Vertex Program");
        let vert_program = compile_dxbc_to_amdil_text(&dll, gsvs.dxbc, |amdil_text| {
            println!("{}", std::str::from_utf8(&amdil_text).unwrap());
            disassemble_amdil_text(amdil_text)
        })
            .expect("couldn't compile vertex shader")
            .expect("couldn't disassemble vertex shader");
        print_output_depedencies(&vert_program);

        println!("\n\nFragment Program");
        let frag_program = compile_dxbc_to_amdil_text(&dll, gsps.dxbc, |amdil_text| {
            println!("{}", std::str::from_utf8(&amdil_text).unwrap());
            disassemble_amdil_text(amdil_text)
        })
            .expect("couldn't compile frag shader")
            .expect("couldn't disassemble frag shader");
        print_output_depedencies(&frag_program);
    // }
}
