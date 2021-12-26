pub(crate) mod caniuse;
pub(crate) mod electron;
pub(crate) mod node;

#[doc(hidden)]
#[allow(unused)]
mod browser_name {
    include!(concat!(env!("OUT_DIR"), "/browser_name_atom.rs"));
}
