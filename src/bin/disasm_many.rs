use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use amd_dx_gsa::Atidxx64;
use clap::Parser;
use yk_fxo_disasm::disasm::print_output_depedencies;
use yk_fxo_disasm::yk::parse_gsfx;

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
    report_path: PathBuf,
}

fn read_fxo(dll: &Atidxx64, fxo_path: std::path::PathBuf) {
    let fxo = std::fs::read(fxo_path).expect("couldn't read fxo file");

    let (_, (gsvs, gsps)) = parse_gsfx(&fxo).expect("couldn't parse fxo file");

    compile_dxbc_to_amdil_text(dll, gsvs.dxbc, disassemble_amdil_text)
        .expect("couldn't compile vertex shader")
        .expect("couldn't disassemble vertex shader");
    // print_output_depedencies(&vert_program);

    // println!("\n\nFragment Program");
    compile_dxbc_to_amdil_text(dll, gsps.dxbc, disassemble_amdil_text)
        .expect("couldn't compile frag shader")
        .expect("couldn't disassemble frag shader");
    // print_output_depedencies(&frag_program);
}

fn main() {
    let args = Args::parse();

    let dll = unsafe { Atidxx64::try_load_lib_from(args.dll_path).expect("no library found") };

    let fxos = std::fs::read_dir(args.fxo_dir)
        .expect("couldn't open directory")
        .filter_map(|file| {
            match file {
                Ok(file) => {
                    if file.file_name().to_str()?.ends_with(".fxo") {
                        Some(file)
                    } else {
                        None
                    }
                },
                Err(_) => None,
            }
        });

    let mut report = std::fs::File::create(args.report_path).expect("couldn't open report file");

    let mut successes = vec![];
    let mut failures: HashMap<String, Vec<String>> = HashMap::new();

    for file in fxos {
        let file_path = file.path();
        let file_name = file.file_name().to_str().unwrap().to_owned();
        let res = std::panic::catch_unwind(|| {
            read_fxo(&dll, file_path);
        });
        match res {
            Ok(_) => {
                // report.write_fmt(format_args!("{file_name}\nSUCCESS\n\n")).unwrap();
                successes.push(file_name);
            },
            Err(e) => {
                let err_msg = {
                    match e.downcast::<String>() {
                        Ok(v) => *v,
                        Err(e) => match e.downcast::<&str>() {
                            Ok(v) => v.to_string(),
                            _ => "Unknown Source of Error".to_owned()
                        }
                    }
                };

                // report.write_fmt(format_args!("{file_name}\nFAILURE\n{err_msg}\n\n")).unwrap();
                match failures.get_mut(&err_msg) {
                    Some(associated_files) => associated_files.push(file_name),
                    None => {
                        failures.insert(err_msg, vec![file_name]);
                    }
                }
            },
        }
    }

    report.write_fmt(format_args!("\n\nSUMMARY\nSuccesses: {}\n\n", successes.len())).unwrap();
    for file_name in successes {
        report.write_fmt(format_args!("{file_name}\n")).unwrap();
    }
    report.write_fmt(format_args!("\nFailures:\n")).unwrap();
    for (err_msg, file_names) in failures {
        report.write_fmt(format_args!("\n{err_msg}\n")).unwrap();
        for file_name in file_names {
            report.write_fmt(format_args!("{file_name}\n")).unwrap();   
        }
    }
}
