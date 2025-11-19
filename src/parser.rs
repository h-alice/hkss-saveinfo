
// nom
use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::{digit0, digit1};
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::combinator::{eof, opt, peek, recognize};
use nom::{IResult, Parser, multi::many_till};

use crate::save_info_struct::SaveNameInfo;

/// Parses internal tag
/// 
/// The excepted internal tag pattern is a string, enclosed in two 
/// "double underscores" (`__`) symbols.
/// 
/// ```rust
/// assert_eq!(parser("__some_attr__"), Ok(("", "some_attr")));
/// assert_eq!(parser("__sometag__user2.dat"), Ok(("user2.dat", "sometag")));
/// ```
fn parse_tag_internal(input: &str) -> IResult<&str, &str> {

    let (input, _) = tag("__")(input)?;
    let (input, internal_tag) = take_until("__")(input)?;
    let (input, _) = tag("__")(input)?;

    return Ok((input, internal_tag));
}


/// Parse backup id from file suffix
/// 
/// ```rust, no-run
/// assert_eq!(parse_suffix_bak(".bak123"), Ok(("", "123")));
/// assert_eq!(parse_suffix_bak(".bak"), Ok(("", "")));
/// ```
fn parse_suffix_bak(input: &str) -> IResult<&str, &str> {

    let (input, _) = tag(".bak")(input)?;
    let (input, bak_id) = digit0(input)?;
    let (input, _) = eof(input)?;
    Ok((input, bak_id))
}


/// Parses suffix and optional backup id
/// 
/// ```rust, no-run
/// assert_eq!(parse_suffix(".dat"), Ok(("", None)));
/// assert_eq!(parse_suffix(".dat.bak"), Ok(("", Some(""))));
/// assert_eq!(parse_suffix(".dat.bak123"), Ok(("", Some("123"))));
/// ```
fn parse_suffix(input: &str) -> IResult<&str, Option<&str>> {

    let (input, _) = tag(".dat")(input)?;
    let (input, bak_id) = opt(parse_suffix_bak).parse(input)?;
    let (input, _) = eof(input)?;
    return Ok((input, bak_id));
}

/// Parse the version tag
/// 
/// ```rust, no-run
/// assert_eq!(parse_version("_1.0.28891"), Ok(("", "1.0.28891"))); // HKSS version
/// assert_eq!(parse_version("_1.2.3.28891"), Ok(("", "1.2.3.28891"))); // legacy HKversion
/// ```
fn parse_version(input: &str) -> IResult<&str, &str> {

    let (input, _) = tag("_")(input)?;

    let (input, version_tag) = 
        recognize(separated_list0(tag("."), digit1)).parse(input)?;

    return Ok((input, version_tag));
}

/// Parses user tag
/// 
/// ```rust, no-run
/// assert_eq!(parse_user_tag("user1.dat"), Ok((".dat", "1"))); // basic case.
/// assert_eq!(parse_user_tag("user4_1.0.28891.dat"), Ok(("_1.0.28891.dat", "4"))); // with version
/// assert_eq!(parse_user_tag("usera-b_c__d.e.dat"), Ok((".dat", "a-b_c__d.e"))); // different symbols
/// ```
fn parse_user_tag(input: &str) -> IResult<&str, &str> {

    let (input, _) = tag("user")(input)?;

    let (input, recognized_tag)= 
        recognize( // combine taken words to str
            many_till( // take 1 word until sub parser succeed
                take(1usize), // try to take 1 word
                peek(preceded(opt(parse_version), parse_suffix))) // look-ahead without consuming
            ).parse(input)?;

    return Ok((input, recognized_tag));
}

pub fn parse(input: &str) -> IResult<&str, SaveNameInfo> {

    // 1. parse internal tag
    let (input, internal_tag) = opt(parse_tag_internal).parse(input)?;

    // 2. parse user tag
    let (input, user_tag) = parse_user_tag(input)?;

    // 3. parse version
    let (input, ver) = opt(parse_version).parse(input)?;

    // 4. parse the suffix
    let (input, backup) = parse_suffix.parse(input)?;

    Ok((
        input, 
        SaveNameInfo {
            tag: user_tag.to_owned(),
            version: ver.map(|x| x.to_owned()),
            backup_id: backup.map(|x| x.to_owned()),
            internal_tag: internal_tag.map(|x| x.to_owned()),
        }
    ))
}


#[test]
fn test_parse() {

    // inner file name match checker
    fn match_checker(input : &str) -> bool {

        let x = parse(input).unwrap().1;
        let s = x.to_string();

        input == s.as_str()
    }

    // basic cases supported by HK ans HKSS
    assert!(match_checker("user1.dat"));
    assert!(match_checker("user2_1.0.28891.dat"));
    assert!(match_checker("user2.dat.bak123"));

    // long name with all field filled
    assert!(match_checker("__pin__useraaa_bbb-ccc.ddd_1.0.28891.dat.bak123"));
    assert!(match_checker("__aa-bb_cc.dd__useraaa_bbb-ccc.ddd_1.2.3.28891.dat.bak123"));
}

#[test]
fn test_parse_suffix() {

    // backup suffix with id
    assert_eq!(parse_suffix_bak(".bak123"), Ok(("", "123")));
    assert_eq!(parse_suffix_bak(".bak"), Ok(("", "")));

    // parse full suffix section
    assert_eq!(parse_suffix(".dat"), Ok(("", None)));
    assert_eq!(parse_suffix(".dat.bak"), Ok(("", Some(""))));
    assert_eq!(parse_suffix(".dat.bak123"), Ok(("", Some("123"))));
    assert_eq!(parse_suffix("err"), Err(nom::Err::Error(nom::error::Error::new("err", nom::error::ErrorKind::Tag))));

}

#[test]
fn test_parser_user_tag() {

    // Part 1: numerical user tag.
    assert_eq!(parse_user_tag("user1.dat"), Ok((".dat", "1"))); // basic case.
    assert_eq!(parse_user_tag("user4_1.0.28891.dat"), Ok(("_1.0.28891.dat", "4"))); // with version
    assert_eq!(parse_user_tag("user1.dat.bak"), Ok((".dat.bak", "1"))); // with backup
    assert_eq!(parse_user_tag("user1.dat.bak123"), Ok((".dat.bak123", "1"))); // with backup id
    assert_eq!(parse_user_tag("user4_1.0.28891.dat.bak123"), Ok(("_1.0.28891.dat.bak123", "4"))); // with version + backup id

    
    // Part 2: general user tag.
    assert_eq!(parse_user_tag("userTest.dat"), Ok((".dat", "Test")));
    assert_eq!(parse_user_tag("usera-b_c__d.e.dat"), Ok((".dat", "a-b_c__d.e"))); // different symbols
    assert_eq!(parse_user_tag("usera-b_c__d.e_1.0.28891.dat"), Ok(("_1.0.28891.dat", "a-b_c__d.e"))); // with version
    assert_eq!(parse_user_tag("usera-b_c__d.e.dat.bak"), Ok((".dat.bak", "a-b_c__d.e"))); // with backup
    assert_eq!(parse_user_tag("usera-b_c__d.e.dat.bak123"), Ok((".dat.bak123", "a-b_c__d.e"))); // with backup id
    assert_eq!(parse_user_tag("usera-b_c__d.e_1.0.28891.dat.bak123"), Ok(("_1.0.28891.dat.bak123", "a-b_c__d.e"))); // with version + backup id

    assert_eq!(parse_user_tag("user1.dat.dat"), Ok((".dat", "1.dat"))); // an extreme case, `1.dat` as user tag.
}