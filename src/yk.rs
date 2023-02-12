use nom::{
    bytes::complete::{tag, take},
    error::{ErrorKind, ParseError},
    number::complete::{le_u16, le_u32},
    sequence::tuple,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum YkGfxError<I> {
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for YkGfxError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        YkGfxError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

pub struct GSVS<'a> {
    pub dxbc: &'a [u8],
}

pub struct GSPS<'a> {
    pub dxbc: &'a [u8],
}

/// Reads the GSFX header
///
/// returns (overall sans GSFX header, (GSVS, GSPS))
pub fn parse_gsfx<'a>(
    overall: &'a [u8],
) -> IResult<&'a [u8], (GSVS<'a>, GSPS<'a>), YkGfxError<&'a [u8]>> {
    let (input, (_magic, _unk1, _unk2, overall_len)) =
        tuple((tag(b"GSFX"), le_u32, le_u32, le_u32))(overall)?;

    let (input, (name_checksum, name)) = tuple((le_u16, take(30_usize)))(input)?;

    let (input, (vs_start, vs_length)) = tuple((le_u32, le_u32))(input)?;
    let (input, (fs_start, fs_length)) = tuple((le_u32, le_u32))(input)?;

    let (vs_start, vs_length, fs_start, fs_length) = (
        vs_start as usize,
        vs_length as usize,
        fs_start as usize,
        fs_length as usize,
    );

    let (_, gsvs) = parse_gsvs(&overall[vs_start..(vs_start + vs_length)])?;
    let (_, gsps) = parse_gsps(&overall[fs_start..(fs_start + fs_length)])?;

    Ok((input, (gsvs, gsps)))
}

pub fn parse_gsvs<'a>(overall: &'a [u8]) -> IResult<&'a [u8], GSVS<'a>, YkGfxError<&'a [u8]>> {
    let (input, (_magic, _unk1, _unk2, dxbc_len)) =
        tuple((tag(b"GSVS"), le_u32, le_u32, le_u32))(overall)?;

    let (input, (_unk3, _unk4, dxbc_offset, dxbc_len)) =
        tuple((le_u32, le_u32, le_u32, le_u32))(input)?;

    let dxbc_offset = dxbc_offset as usize;
    let dxbc_len = dxbc_len as usize;

    Ok((
        input,
        GSVS {
            dxbc: &overall[dxbc_offset..(dxbc_offset + dxbc_len)],
        },
    ))
}

pub fn parse_gsps<'a>(overall: &'a [u8]) -> IResult<&'a [u8], GSPS<'a>, YkGfxError<&'a [u8]>> {
    let (input, (_magic, _unk1, _unk2, dxbc_len)) =
        tuple((tag(b"GSPS"), le_u32, le_u32, le_u32))(overall)?;

    let (input, (_unk3, _unk4, dxbc_offset, dxbc_len)) =
        tuple((le_u32, le_u32, le_u32, le_u32))(input)?;

    let dxbc_offset = dxbc_offset as usize;
    let dxbc_len = dxbc_len as usize;

    Ok((
        input,
        GSPS {
            dxbc: &overall[dxbc_offset..(dxbc_offset + dxbc_len)],
        },
    ))
}
