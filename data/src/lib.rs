pub mod caniuse;
pub mod electron;
pub mod node;
mod utils;

#[doc(hidden)]
pub fn decode_browser_name(id: u8) -> &'static str {
    match id {
        1 => "ie",
        2 => "edge",
        3 => "firefox",
        4 => "chrome",
        5 => "safari",
        6 => "opera",
        7 => "ios_saf",
        8 => "op_mini",
        9 => "android",
        10 => "bb",
        11 => "op_mob",
        12 => "and_chr",
        13 => "and_ff",
        14 => "ie_mob",
        15 => "and_uc",
        16 => "samsung",
        17 => "and_qq",
        18 => "baidu",
        19 => "kaios",
        _ => unreachable!("cannot recognize browser id"),
    }
}
