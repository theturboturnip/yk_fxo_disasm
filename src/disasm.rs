use std::{collections::{HashSet, HashMap}, thread::current, panic::PanicInfo};

use turnip_gfx_disasm::{
    abstract_machine::{analysis::dependency::ScalarDependencies, VMName, display::DisplayVec, vector::VectorOf},
    amdil_text::{AMDILDecodeError, AMDILDecoder, AMDILProgram},
    // rdna2::{vm::RDNA2DataRef, RDNA2DecodeError, RDNA2Decoder, RDNA2Program},
    Decoder, Program, hlsl::{compat::{HLSLCompatibleAbstractVM, program_to_hlsl}, display::DWrap, kinds::{HLSLKindBitmask, HLSLKind}, HLSLScalar, HLSLVector, vm::HLSLAbstractVM}
};

// pub fn disassemble_rdna2(rdna2: &[u8]) -> Result<RDNA2Program, RDNA2DecodeError> {
//     RDNA2Decoder::new().decode(rdna2)
// }

pub fn print_output_depedencies<T: HLSLCompatibleAbstractVM>(program: &impl Program<T>) {
    let program_compat = program_to_hlsl::<T, _>(program);

    let mut resolver = ScalarDependencies::<HLSLAbstractVM>::new();
    for action in program_compat.actions() {
        resolver.accum_action(action, &HashSet::new());
    }

    println!("Inputs and Outputs:");
    for r in program_compat.io_declarations() {
        println!("\t{:?}", r);
    }

    for dependent in resolver.discard_dependencies() {
        println!("discard depends on {}", DWrap((dependent, HLSLKindBitmask::all().into())))
    }

    let mut out_deps: Vec<_> = resolver.dependents().into_iter().filter_map(|(out, deps)| {
        if out.is_output() {
            let mut v = deps.iter().collect::<Vec<&HLSLScalar>>();
            v.sort();
            let mut lits = vec![];
            let mut vecs = vec![];
            let mut current_vec = None;

            for s in v{
                match s {
                    HLSLScalar::Component(reg, _) => {
                        match &mut current_vec {
                            None => {
                                current_vec = Some((reg, vec![s.clone()]))
                            }
                            Some((other_reg, other_scalars)) if reg != *other_reg => {
                                vecs.push((
                                    HLSLVector::new(&other_scalars).unwrap(),
                                    other_reg.toplevel_kind()
                                ));
                                current_vec = Some((reg, vec![s.clone()]))
                            }
                            Some((_this_reg, sibling_scalars)) => {
                                sibling_scalars.push(s.clone())
                            }
                        }
                    },
                    HLSLScalar::Literal(_) => lits.push(s),
                }
            }

            if let Some((reg, scalars)) = current_vec {
                vecs.push((HLSLVector::new(&scalars).unwrap(), reg.toplevel_kind()))
            }
            Some(
                (out, lits, vecs)
            )
        } else {
            None
        }
    }).collect();
    out_deps.sort_by(|(out1, ..), (out2, ..)| out1.partial_cmp(out2).unwrap());

    for (out, lits, vecs) in out_deps {
        println!("{} depends on {}; {}", 
            DWrap((out, HLSLKindBitmask::all().into())),
            DisplayVec::Sep { vec: &(vecs.iter().map(|v| DWrap(v)).collect()), sep: ", " },
            DisplayVec::Sep { vec: &(lits.into_iter().map(|l| DWrap((l, HLSLKindBitmask::all().into()))).collect()), sep: ", "}
        )
    }
}

pub fn disassemble_amdil_text(amdil_text: &[u8]) -> Result<AMDILProgram, AMDILDecodeError> {
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