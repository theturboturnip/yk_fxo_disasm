use std::{collections::{HashSet, HashMap}, thread::current, panic::PanicInfo};

use turnip_gfx_disasm::{
    abstract_machine::{analysis::{dependency::ScalarDependencies, variable::disassemble}, VMName, display::DisplayVec, expr::{HLSLVector, HLSLScalar, Reg}},
    amdil_text::{AMDILErrorContext, AMDILDecoder, AMDILProgram},
    // rdna2::{vm::RDNA2DataRef, RDNA2DecodeError, RDNA2Decoder, RDNA2Program},
    Decoder, Program, hlsl::{compat::{HLSLCompatibleAbstractVM, program_to_hlsl}, display::DWrap, kinds::{HLSLKindBitmask, HLSLKind}, vm::HLSLAbstractVM}
};

// pub fn disassemble_rdna2(rdna2: &[u8]) -> Result<RDNA2Program, RDNA2DecodeError> {
//     RDNA2Decoder::new().decode(rdna2)
// }
pub fn analyze_program<T: HLSLCompatibleAbstractVM>(program: &impl Program<T>) {
    let disassembled = disassemble(&program_to_hlsl::<T, _>(program));

    let mut scalar_deps = ScalarDependencies::<HLSLAbstractVM>::new();
    let empty_ctrl_flow = HashSet::new();
    for action in disassembled.actions() {
        scalar_deps.accum_action(action, &empty_ctrl_flow);
    }
}

pub fn print_output_depedencies<T: HLSLCompatibleAbstractVM>(program: &impl Program<T>) {
    let program_compat = disassemble(&program_to_hlsl::<T, _>(program));

    let mut resolver = ScalarDependencies::<HLSLAbstractVM>::new();
    for action in program_compat.actions() {
        resolver.accum_action(action, &HashSet::new());
    }

    println!("Inputs and Outputs:");
    for r in program_compat.io_declarations() {
        println!("\t{:?}", r);
    }

    for dependent in resolver.discard_dependencies {
        println!("discard depends on {:?}", dependent)
    }

    let mut out_deps: Vec<_> = resolver.dependents.iter().filter_map(|(out, deps)| {
        if out.0.is_output() {
            let mut v = deps.iter().collect::<Vec<_>>();
            v.sort();
            let mut vecs = vec![];
            let mut current_vec = None;

            for (written_reg, written_comp, written_kind) in v {
                current_vec = match current_vec {
                    None => {
                        Some((written_reg.clone(), vec![*written_comp]))
                    }
                    Some((other_reg, other_scalars)) if *written_reg != other_reg => {
                        vecs.push(
                            (other_reg, other_scalars)
                        );
                        Some((written_reg.clone(), vec![*written_comp]))
                    }
                    Some((this_reg, mut sibling_scalars)) => {
                        sibling_scalars.push(*written_comp);
                        Some((this_reg, sibling_scalars))
                    }
                }
            }

            if let Some((reg, scalars)) = current_vec {
                vecs.push(
                    (reg, scalars)
                );
            }
            Some(
                (out, vecs)
            )
        } else {
            None
        }
    }).collect();
    out_deps.sort_by(|(out1, ..), (out2, ..)| out1.partial_cmp(out2).unwrap());

    for (out, vecs) in out_deps {
        println!("{:?}.{} depends on {:?}", 
            out.0, out.1,
            vecs
            // DisplayVec::Sep { vec: &(vecs.iter().map(|v| DWrap((v, v.output_kind()))).collect()), sep: ", " },
            // DisplayVec::Sep { vec: &(lits.into_iter().map(|l| DWrap(l))).collect(), sep: ", "}
        )
    }

    println!("PROGRAM TEXT BEGIN");
    for a in program_compat.actions {
        println!("{}", a);
    }
}

pub fn disassemble_amdil_text(amdil_text: &[u8]) -> Result<AMDILProgram, AMDILErrorContext> {
    let amdil_text = std::str::from_utf8(amdil_text).expect("text was invalid utf8");
    AMDILDecoder::new().decode(amdil_text)
}

/*
pub fn resolve_variable_text_dependencies<T: AbstractVM>(program: &impl Program<T>) {
    // Perform a variable pass first
    let mut variable_resolver = VariableAbstractMachine::new();
    for action in program.actions() {
        variable_resolver.accum_action(action);
        for outcome in action.outcomes() {
            match outcome {
                Outcome::Assign { output, inputs, .. } => {
                    println!(
                        "\t{:?} <- {}",
                        output,
                        DebugVec::Prefix {
                            vec: &inputs,
                            prefix: "\n\t\t"
                        }
                    )
                }
                Outcome::Declare(name) => println!("\t{:?} exists", name),

                Outcome::EarlyOut { inputs } => {
                    println!(
                        "EarlyOut[{}]",
                        DebugVec::Prefix {
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
            HLSLSingleVectorName::ShaderOutput { .. } => {
                println!("{} depends on", DWrap(dependent.0));
                let mut inputs: Vec<_> = dependent
                    .1
                    .into_iter()
                    .map(|x| format!("{}", DWrap(x)))
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
*/