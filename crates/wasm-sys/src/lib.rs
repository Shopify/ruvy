#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Re-export Ruby special constants with shorter names for compatibility
pub const RUBY_Qnil: VALUE = ruby_special_consts_RUBY_Qnil as VALUE;
pub const RUBY_Qfalse: VALUE = ruby_special_consts_RUBY_Qfalse as VALUE;
pub const RUBY_Qtrue: VALUE = ruby_special_consts_RUBY_Qtrue as VALUE;
pub const RUBY_Qundef: VALUE = ruby_special_consts_RUBY_Qundef as VALUE;
