use syn::{Expr, ExprBinary, Item, Stmt};
use try_match::match_ok;

pub fn extract_const_asserts(items: &mut Vec<Item>) -> Vec<Item> {
    let new_items = items
        .extract_if(.., |item| match item {
            Item::Const(item) => item.ident == "_",
            _ => false,
        })
        .filter_map(|item| {
            let const_ = match_ok!(item, Item::Const(x))?;
            let block = match_ok!(*const_.expr, Expr::Block(x))?;
            Some(block)
        })
        .flat_map(|block| block.block.stmts)
        .filter_map(|stmt| {
            let expr = match_ok!(stmt, Stmt::Expr(x, _))?;
            let indx = match_ok!(expr, Expr::Index(x))?;
            let ExprBinary { left, right, .. } = match_ok!(*indx.index, Expr::Binary(x))?;
            let macr = syn::parse_quote! {static_assertions::const_assert_eq!(#left,#right);};
            Some(macr)
        })
        .collect::<Vec<_>>();
    new_items
}
