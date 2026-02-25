use bindgen::{
    Builder,
    callbacks::{DeriveTrait, ImplementsTrait, ParseCallbacks},
};

pub fn common_generator() -> Builder {
    bindgen::builder()
        .clang_arg("-fparse-all-comments")
        .generate_comments(true)
        .use_core()
        .raw_line(r"#![allow(non_snake_case)]")
        .raw_line(r"#![allow(non_camel_case_types)]")
        .raw_line(r"#![allow(non_upper_case_globals)]")
        .raw_line(r"#![allow(dead_code)]")
        .raw_line(r"#![allow(unused_imports)]")
        .sort_semantically(true)
        .generate_cstr(true)
        .formatter(bindgen::Formatter::Prettyplease)
}

#[derive(Debug, Default, Clone)]
pub struct MyCallbacks;

impl ParseCallbacks for MyCallbacks {
    fn blocklisted_type_implements_trait(
        &self,
        _: &str,
        derive_trait: DeriveTrait,
    ) -> Option<ImplementsTrait> {
        match derive_trait {
            DeriveTrait::Debug | DeriveTrait::Copy => Some(ImplementsTrait::Yes),
            _ => None,
        }
    }

    fn process_comment(&self, comment: &str) -> Option<String> {
        if comment.contains("is a preprocessor guard") {
            return Some(String::from(""));
        }
        None
    }
}
