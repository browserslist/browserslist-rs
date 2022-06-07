pub(crate) mod caniuse;
pub(crate) mod electron;
pub(crate) mod node;

#[doc(hidden)]
#[allow(unused)]
pub(crate) mod browser_name {
    include!(concat!(env!("OUT_DIR"), "/browser_name_atom.rs"));

    pub fn decode_browser_name(id: u8) -> BrowserNameAtom {
        match id {
            1 => browser_name_atom!("ie"),
            2 => browser_name_atom!("edge"),
            3 => browser_name_atom!("firefox"),
            4 => browser_name_atom!("chrome"),
            5 => browser_name_atom!("safari"),
            6 => browser_name_atom!("opera"),
            7 => browser_name_atom!("ios_saf"),
            8 => browser_name_atom!("op_mini"),
            9 => browser_name_atom!("android"),
            10 => browser_name_atom!("bb"),
            11 => browser_name_atom!("op_mob"),
            12 => browser_name_atom!("and_chr"),
            13 => browser_name_atom!("and_ff"),
            14 => browser_name_atom!("ie_mob"),
            15 => browser_name_atom!("and_uc"),
            16 => browser_name_atom!("samsung"),
            17 => browser_name_atom!("and_qq"),
            18 => browser_name_atom!("baidu"),
            19 => browser_name_atom!("kaios"),
            _ => unreachable!("cannot recognize browser id"),
        }
    }
}
