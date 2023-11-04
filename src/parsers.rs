use nom::IResult;
use nom::bytes::complete;




pub fn file_tag(i: &[u8]) -> IResult<&[u8],&[u8]> {
    complete::tag([77u8, 65u8, 90u8, 69u8])(i)
}
