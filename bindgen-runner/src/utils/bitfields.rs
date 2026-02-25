use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
};

use memchr::memmem;
use syn::Item;
use try_match::match_ok;

use crate::utils::{Platform, clangs::{
    Clang, FieldDecl, FieldType, IntegerLiteral, Node, RecordDecl, clang_dump_ast,
    clang_preprocess,
}, common_generator, MyCallbacks};

pub type Bitfields<'ast, 'json> = HashMap<&'ast Node<'json>, Vec<Vec<&'ast Node<'json>>>>;

pub fn find_bitfields<'a, 'json>(tu: &'a Node<'json>) -> Bitfields<'a, 'json> {
    let mut bitfields = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back(tu);
    while let Some(n) = queue.pop_front() {
        match n.kind {
            Clang::RecordDecl(RecordDecl { .. }) => {
                let mut size_accum = 0;
                let mut storage = None;
                for n_field in &n.inner {
                    match n_field.kind {
                        Clang::FieldDecl(FieldDecl {
                            is_bitfield,
                            r#type:
                                FieldType {
                                    qual_type,
                                    desugared_qual_type,
                                },
                            ..
                        }) => {
                            if is_bitfield.unwrap_or(false) {
                                assert!(
                                    qual_type == "unsigned int"
                                        || desugared_qual_type.is_some_and(|t| t == "unsigned int")
                                );
                                let n_int_lit = &n_field.inner[0].inner[0];
                                let Clang::IntegerLiteral(IntegerLiteral { value: width, .. }) =
                                    n_int_lit.kind
                                else {
                                    unreachable!()
                                };
                                assert!(width <= 32);
                                assert!(storage.is_some() || width != 0);
                                let stores = bitfields.entry(n).or_insert(vec![]);
                                if width + size_accum > 32 {
                                    stores.extend(storage.take());
                                    size_accum = 0;
                                }
                                storage.get_or_insert(vec![]).push(n_field);
                                size_accum += width;
                                if width == 0 {
                                    stores.extend(storage.take());
                                    size_accum = 0;
                                }
                            } else {
                                bitfields
                                    .entry(n)
                                    .and_modify(|ss| ss.extend(storage.take()));
                                size_accum = 0;
                            }
                        }
                        _ => queue.extend(&n_field.inner),
                    }
                }
                bitfields
                    .entry(n)
                    .and_modify(|ss| ss.extend(storage.take()));
            }
            _ => queue.extend(&n.inner),
        }
    }
    bitfields
}

pub fn replace_bitfield_with_single_field(src: &mut Vec<u8>, bitfields: &Bitfields) {
    let mut segments = bitfields
        .iter()
        .flat_map(|(_rec, stores)| stores.into_iter().enumerate())
        .map(|(i, storage)| {
            let end_i = {
                let last_int_lit = &storage[storage.len() - 1].inner[0].inner[0];
                let Clang::IntegerLiteral(int_lit) = &last_int_lit.kind else {
                    unreachable!()
                };
                let end_loc = int_lit.range.end.spelling_loc.as_ref().unwrap();
                end_loc.offset + end_loc.tok_len
            };
            let first_field_i = {
                let Clang::FieldDecl(first_field) = &storage[0].kind else {
                    unreachable!()
                };
                let first_name = first_field.name.expect("unnamed field is unexpected");
                let begin_i = {
                    let begin_loc = first_field.range.begin.spelling_loc.as_ref().unwrap();
                    begin_loc.offset
                };
                let code_seg = &src[begin_i..end_i];
                let name = first_name.as_bytes();
                let find = memmem::find(code_seg, name).unwrap();
                find + begin_i
            };
            ((first_field_i, end_i), i)
        })
        .collect::<Vec<_>>();
    segments.sort_unstable_by_key(|&((a, _), _)| Reverse(a));
    for ((a, b), i) in segments {
        src.splice(a..b, format!("__bitfield_storage_{i}").bytes());
    }
}

pub type BitfieldInfo = HashMap<String, Vec<Vec<(String, u64)>>>;

pub fn get_bitfield_info(bitfields: &Bitfields) -> BitfieldInfo {
    bitfields
        .iter()
        .map(|(rec, ss)| {
            let Clang::RecordDecl(rec) = &rec.kind else {
                unreachable!()
            };
            let rec_name = rec.name.expect("unnamed struct is unexpected");
            let storages_info = ss
                .iter()
                .map(|s| {
                    s.iter()
                        .map(|n_field| {
                            let Clang::FieldDecl(field) = &n_field.kind else {
                                unreachable!()
                            };
                            let n_int_lit = &n_field.inner[0].inner[0];
                            let Clang::IntegerLiteral(IntegerLiteral { value: width, .. }) =
                                n_int_lit.kind
                            else {
                                unreachable!()
                            };
                            let field_name = field.name.expect("unnamed field is unexpected");
                            (field_name.to_owned(), width)
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            (rec_name.to_owned(), storages_info)
        })
        .collect::<HashMap<_, _>>()
}

pub fn generate_bitfield_types(
    platform: &Platform,
) -> anyhow::Result<Vec<(syn::ItemStruct, Vec<Vec<(String, u64)>>)>> {
    let prepr_header_src = clang_preprocess(platform)?;
    let ast_json = clang_dump_ast(prepr_header_src.as_bytes())?;
    let ast = serde_json::from_str(&ast_json)?;
    let bitfields = find_bitfields(&ast);
    let mut moded_src = prepr_header_src.into_bytes();
    replace_bitfield_with_single_field(&mut moded_src, &bitfields);
    let moded_src: String = moded_src.try_into()?;
    let mut infos = get_bitfield_info(&bitfields);
    let moded_types_re = infos.keys().fold(String::new(), |acc, s| acc + s + "|");
    let binds = common_generator()
        .header_contents("moded_vk_all.h", &moded_src)
        .allowlist_recursively(false)
        .allowlist_type(&moded_types_re.trim_matches('|'))
        .parse_callbacks(Box::new(MyCallbacks))
        .layout_tests(false)
        .generate()?;
    let tree = syn::parse_file(&binds.to_string())?;
    let structs = tree
        .items
        .into_iter()
        .filter_map(|item| match_ok!(item, Item::Struct(x)))
        .map(|item| {
            let name = item.ident.to_string();
            let info = infos.remove(&name).expect("missing Struct");
            (item, info)
        })
        .collect::<Vec<_>>();
    Ok(structs)
}
