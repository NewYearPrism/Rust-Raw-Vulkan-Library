use std::{
    env,
    ffi::OsString,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::LazyLock,
};

use anyhow::anyhow;
use serde::Deserialize;

use crate::utils::Platform;
use crate::VULKAN_INCLUDE;

pub static CLANG_PATH: LazyLock<OsString> =
    LazyLock::new(|| env::var_os("CLANG_PATH").unwrap_or("clang".into()));

pub const DUMP_AST_ARGS: &[&str] = &["-xc", "-Xclang", "-ast-dump=json", "-fsyntax-only"];
pub const PREPROCESS_ARGS: &[&str] = &["-xc", "-E", "-C", "-fparse-all-comments"];

pub type Node<'json> = clang_ast::Node<Clang<'json>>;

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDecl<'json> {
    pub name: Option<&'json str>,
    pub is_bitfield: Option<bool>,
    pub r#type: FieldType<'json>,
    pub loc: clang_ast::SourceLocation,
    pub range: clang_ast::SourceRange,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDecl<'json> {
    pub name: Option<&'json str>,
    pub complete_definition: Option<bool>,
}

#[serde_with::serde_as]
#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegerLiteral {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub value: u64,
    pub range: clang_ast::SourceRange,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Clang<'json> {
    RecordDecl(#[serde(borrow)] RecordDecl<'json>),
    FieldDecl(FieldDecl<'json>),
    IntegerLiteral(IntegerLiteral),
    Other { kind: clang_ast::Kind },
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldType<'json> {
    pub qual_type: &'json str,
    pub desugared_qual_type: Option<&'json str>,
}

pub fn clang_preprocess(platform: &Platform) -> anyhow::Result<String> {
    let clang_path: &Path = CLANG_PATH.as_ref();
    let vulkan_include: &Path = VULKAN_INCLUDE.as_ref();
    let output = Command::new(clang_path)
        .args(PREPROCESS_ARGS)
        .arg(vulkan_include.join("vulkan/vulkan.h"))
        .args(["-I".as_ref(), vulkan_include])
        .arg("-DVK_NO_PROTOTYPES")
        .arg(format!("--target={}", &platform.triplet))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .output()?;
    let preproc_output = output
        .status
        .success()
        .then(|| output.stdout)
        .ok_or(anyhow!("clang command failed"))?;
    let src = String::from_utf8(preproc_output)?;
    Ok(src)
}

pub fn clang_dump_ast(input: &[u8]) -> anyhow::Result<String> {
    let platform = Platform::_WIN64;
    let clang_path: &Path = CLANG_PATH.as_ref();
    let child = Command::new(clang_path)
        .args(DUMP_AST_ARGS)
        .arg(format!("--target={}", &platform.triplet))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        child.stdin.unwrap().write_all(&input)?;
    }
    let mut json = String::with_capacity(4 * 1024 * 1024);
    child.stdout.unwrap().read_to_string(&mut json)?;
    Ok(json)
}

pub fn preprocess_guards(platform: &Platform) -> anyhow::Result<Vec<String>> {
    let pre = clang_preprocess(platform)?;
    let res = pre
        .lines()
        .filter(|l| l.contains("is a preprocessor guard"))
        .flat_map(|l| l.split(' '))
        .filter(|w| w.starts_with("VK_") || w.starts_with("vulkan_"))
        .map(str::to_owned)
        .collect();
    Ok(res)
}
