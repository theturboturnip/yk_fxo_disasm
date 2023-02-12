use amd_dx_gsa::{
    amd_isa_devices::FIRST_RDNA2_ASIC, dxbc::get_shader_bytecode, Atidxx64, ShaderCompileError,
};
use object::{Object, ObjectSection};

pub fn compile_dxbc_to_rdna2<'a, T, F: Fn(&[u8]) -> T>(
    dll: &Atidxx64,
    dxbc: &'a [u8],
    callback: F,
) -> Result<T, ShaderCompileError> {
    let (_, bytecode) = get_shader_bytecode(dxbc).expect("couldn't extract bytecode from DXBC");
    dll.inspect_compiled_shader(
        FIRST_RDNA2_ASIC,
        amd_dx_gsa::AmdDxGsaShaderSource::DxAsmBinary(bytecode),
        vec![],
        |elf| {
            let obj_file = object::File::parse(elf).expect("invalid ELF produced by atidxx64.dll");
            println!(
                "{}",
                String::from_utf8(
                    obj_file
                        .section_by_name(".amdil_disassembly")
                        .unwrap()
                        .data()
                        .unwrap()
                        .to_vec()
                )
                .unwrap()
            );
            let section = obj_file.section_by_name(".text").expect("no text section");
            callback(section.data().expect(".text has no data"))
        },
    )
}

pub fn compile_dxbc_to_amdil_text<'a, T, F: Fn(&[u8]) -> T>(
    dll: &Atidxx64,
    dxbc: &'a [u8],
    callback: F,
) -> Result<T, ShaderCompileError> {
    let (_, bytecode) = get_shader_bytecode(dxbc).expect("couldn't extract bytecode from DXBC");
    dll.inspect_compiled_shader(
        FIRST_RDNA2_ASIC,
        amd_dx_gsa::AmdDxGsaShaderSource::DxAsmBinary(bytecode),
        vec![],
        |elf| {
            let obj_file = object::File::parse(elf).expect("invalid ELF produced by atidxx64.dll");
            println!(
                "{}",
                String::from_utf8(
                    obj_file
                        .section_by_name(".amdil_disassembly")
                        .unwrap()
                        .data()
                        .unwrap()
                        .to_vec()
                )
                .unwrap()
            );
            let section = obj_file
                .section_by_name(".amdil_disassembly")
                .expect("no amdil_disassembly section");
            callback(section.data().expect(".amdil_disassembly has no data"))
        },
    )
}
