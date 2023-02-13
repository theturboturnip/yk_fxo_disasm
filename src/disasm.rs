use turnip_gfx_disasm::{
    abstract_machine::analysis::{
        dependency::ScalarDependencies, variable::VariableAbstractMachine,
    },
    abstract_machine::display::DisplayVec,
    abstract_machine::{ScalarAction, ScalarOutcome},
    amdil_text::{AMDILDecodeError, AMDILDecoder, AMDILProgram},
    hlsl::{display::DWrap, HLSLVectorName},
    rdna2::{vm::RDNA2DataRef, RDNA2DecodeError, RDNA2Decoder, RDNA2Program},
    Decoder, Program,
};

pub fn disassemble_rdna2(rdna2: &[u8]) -> Result<RDNA2Program, RDNA2DecodeError> {
    RDNA2Decoder::new().decode(rdna2)
}

pub fn print_output_depedencies(program: &RDNA2Program) {
    let mut resolver = ScalarDependencies::new();
    for action in program {
        resolver.accum_action(action.as_ref());
    }

    for dependent in resolver.dependents() {
        match dependent.0 {
            RDNA2DataRef::Output { .. } => {
                println!("{:?} depends on {:?}", dependent.0, dependent.1)
            }
            _ => {}
        }
    }
}

pub fn disassemble_amdil_text(amdil_text: &[u8]) -> Result<AMDILProgram, AMDILDecodeError> {
    let amdil_text = std::str::from_utf8(amdil_text).expect("text was invalid utf8");
    AMDILDecoder::new().decode(amdil_text)
}

pub fn resolve_amdil_text_dependencies(program: AMDILProgram) {
    // Perform a variable pass first
    let mut variable_resolver = VariableAbstractMachine::new();
    for action in program.actions() {
        variable_resolver.accum_action(action);
        for outcome in action.outcomes() {
            match outcome {
                ScalarOutcome::Dependency { output, inputs } => {
                    println!(
                        "\t{} <- {}",
                        output,
                        DisplayVec::Prefix {
                            vec: &inputs,
                            prefix: "\n\t\t"
                        }
                    )
                }
                ScalarOutcome::Declaration { name, value } => match value {
                    Some(v) => println!("\t{} <- {}", name, v),
                    None => println!("\t{} exists", name),
                },
                ScalarOutcome::EarlyOut { inputs } => {
                    println!(
                        "EarlyOut[{}]",
                        DisplayVec::Prefix {
                            vec: &inputs,
                            prefix: "\n\t\t"
                        }
                    )
                }
            }
        }
    }

    // Then do dependency resolution - hopefully by that point the types will be done
    let mut dependency_resolver = ScalarDependencies::new();
    for action in variable_resolver.actions() {
        println!("{}", action);
        dependency_resolver.accum_action(&action);
    }

    for dependent in dependency_resolver.dependents() {
        match &dependent.0 .0.vector_name {
            HLSLVectorName::ShaderOutput { .. } => {
                println!("{} {} depends on", dependent.0 .0.kind, DWrap(dependent.0));
                let mut inputs: Vec<_> = dependent
                    .1
                    .into_iter()
                    .map(|x| format!("{} {}", x.kind, DWrap(&x.data)))
                    .collect();
                inputs.sort();
                for input in inputs {
                    println!("\t{input}")
                }
            }
            _ => {}
        }
    }
}
