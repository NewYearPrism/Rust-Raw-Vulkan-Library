mod utils;

use std::{env, ffi::OsString, fs, path::Path, sync::LazyLock};

use quote::ToTokens;

use crate::utils::*;

pub static VULKAN_INCLUDE: LazyLock<OsString> = LazyLock::new(|| {
    env::var_os("VULKAN_INCLUDE").expect("MUST specify VULKAN_INCLUDE enviroment variable")
});

#[allow(dead_code)]
fn gen_const_asserts() -> anyhow::Result<()> {
    for platform in Platform::TYPICAL {
        let vulkan_include: &Path = VULKAN_INCLUDE.as_ref();
        let code = common_generator()
            .clang_args(["-I", vulkan_include.to_str().unwrap()])
            .clang_arg("-DVK_NO_PROTOTYPES")
            .clang_arg(format!("--target={}", &platform.triplet))
            .header(vulkan_include.join("vulkan/vulkan.h").to_string_lossy())
            .allowlist_recursively(true)
            .allowlist_file(r".*vulkan.*\.h")
            .prepend_enum_name(false)
            .sort_semantically(false)
            .generate()?
            .to_string();
        let mut syntax_tree = syn::parse_file(&code)?;
        let const_asserts = extract_const_asserts(&mut syntax_tree.items);
        let const_asserts_tree = syn::File {
            shebang: Default::default(),
            attrs: Default::default(),
            items: const_asserts,
        };
        let const_asserts_code = const_asserts_tree.to_token_stream().to_string();
        let reformatted_code = rustfmt_huge_width(&const_asserts_code)?;
        fs::write(
            &format!("output/layout_assert_{}.rs", platform.mod_name()),
            reformatted_code,
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn gen_core() -> anyhow::Result<()> {
    for platform in Platform::TYPICAL {
        let vulkan_include: &Path = VULKAN_INCLUDE.as_ref();
        let code = common_generator()
            .clang_args(["-I", vulkan_include.to_str().unwrap()])
            .clang_arg("-DVK_NO_PROTOTYPES")
            .clang_arg(format!("--target={}", &platform.triplet))
            .header(vulkan_include.join("vulkan/vulkan.h").to_string_lossy())
            .allowlist_recursively(true)
            .allowlist_file(r".*vulkan.*\.h")
            .prepend_enum_name(false)
            .generate()?
            .to_string();
        let mut syntax_tree = syn::parse_file(&code)?;
        let _ = extract_const_asserts(&mut syntax_tree.items);
        let code = prettyplease::unparse(&syntax_tree);
        fs::write(
            format!("output/vulkan_core_{}.rs", platform.mod_name()),
            code,
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn gen_win64_core() -> anyhow::Result<()> {
    let platform = Platform::_WIN64;
    let vulkan_include: &Path = VULKAN_INCLUDE.as_ref();
    let bf = generate_bitfield_types(&platform)?;
    let block_types = bf
        .iter()
        .map(|(i, _)| i.ident.to_string())
        .collect::<Vec<_>>();
    let block_type_re = block_types.join("|");
    let guards = preprocess_guards(&platform)?;
    let block_var_re = guards.join("|");
    let code = common_generator()
        .clang_args(["-I", vulkan_include.to_str().unwrap()])
        .clang_arg("-DVK_NO_PROTOTYPES")
        .clang_arg(format!("--target={}", &platform.triplet))
        .header(vulkan_include.join("vulkan/vulkan.h").to_string_lossy())
        .allowlist_recursively(true)
        .allowlist_file(r".*vulkan.*\.h")
        .blocklist_type(&block_type_re)
        .blocklist_var(&block_var_re)
        .parse_callbacks(Box::new(MyCallbacks))
        .prepend_enum_name(false)
        .generate()?
        .to_string();
    let mut syntax_tree = syn::parse_file(&code)?;
    let _ = extract_const_asserts(&mut syntax_tree.items);
    syntax_tree
        .items
        .extend(bf.into_iter().map(|(i, _)| i.into()));
    let code = prettyplease::unparse(&syntax_tree);
    fs::write("output/vulkan_core.rs", code)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    gen_const_asserts()?;
    gen_core()?;
    gen_win64_core()?;
    Ok(())
}
