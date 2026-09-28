//! Collects the language features, the imports, and the paths, macros, and method calls
//! used by a source file.
//!
//! The visitor does not resolve names. It records what is written, together with the
//! imports of the file and the names defined by the crate itself, so the usages can be
//! resolved once all files are visited.

use crate::LanguageFeature;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// Macros which take a format string, which may capture arguments implicitly.
const FORMAT_MACROS: [&str; 9] = [
    "format",
    "format_args",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "panic",
];

/// A usage of something which may be an item of the standard library.
#[derive(Debug)]
pub enum Usage {
    /// A path, e.g. `Option::is_some_and`, `fs::chown`, or `std::hint::black_box`.
    Path(Vec<String>),
    /// A macro invocation, e.g. `todo!()`.
    Macro(Vec<String>),
    /// A method call, e.g. `x.is_some_and(f)`.
    Method(String),
}

/// The names defined by the scanned crate.
#[derive(Debug, Default)]
pub struct LocalNames {
    pub functions: HashSet<String>,
    pub macros: HashSet<String>,
    /// Types, traits, and modules.
    pub parents: HashSet<String>,
}

impl LocalNames {
    pub fn extend(&mut self, other: &LocalNames) {
        self.functions.extend(other.functions.iter().cloned());
        self.macros.extend(other.macros.iter().cloned());
        self.parents.extend(other.parents.iter().cloned());
    }
}

/// The imports of a file.
///
/// Imports are collected for the whole file, regardless of the block or module they are
/// declared in.
#[derive(Debug, Default)]
pub struct Imports {
    /// The imported names, e.g. `fs` for `use std::os::unix::fs;`, by the path they refer to.
    pub names: HashMap<String, Vec<String>>,
    /// The paths of glob imports, e.g. `std::sync` for `use std::sync::*;`.
    pub globs: Vec<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct Findings {
    pub features: Vec<(LanguageFeature, usize)>,
    pub usages: Vec<(Usage, usize)>,
    pub imports: Imports,
    pub local_names: LocalNames,
}

#[derive(Debug, Default)]
pub struct Visitor {
    pub findings: Findings,
}

impl Visitor {
    fn feature(&mut self, feature: LanguageFeature, node: &impl Spanned) {
        self.findings.features.push((feature, line(node)));
    }

    fn usage(&mut self, usage: Usage, node: &impl Spanned) {
        self.findings.usages.push((usage, line(node)));
    }

    fn path(&mut self, path: &syn::Path) {
        self.usage(Usage::Path(segments(path)), path);
    }

    fn macro_invocation(&mut self, mac: &syn::Macro) {
        let Some(name) = mac.path.segments.last().map(|s| s.ident.to_string()) else {
            return;
        };

        if FORMAT_MACROS.contains(&name.as_str()) && has_implicit_format_argument(mac) {
            self.feature(LanguageFeature::ImplicitFormatArguments, mac);
        }

        self.usage(Usage::Macro(segments(&mac.path)), mac);
    }

    fn use_tree(&mut self, prefix: &mut Vec<String>, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.use_tree(prefix, &path.tree);
                prefix.pop();
            }
            syn::UseTree::Name(name) if name.ident == "self" => {
                // `use std::io::{self}` imports `std::io` as `io`
                if let Some(last) = prefix.last() {
                    self.import(last.clone(), prefix.clone(), &name.ident);
                }
            }
            syn::UseTree::Name(name) => {
                let mut path = prefix.clone();
                path.push(name.ident.to_string());
                self.import(name.ident.to_string(), path, &name.ident);
            }
            syn::UseTree::Rename(rename) => {
                let mut path = prefix.clone();
                path.push(rename.ident.to_string());
                self.import(rename.rename.to_string(), path, &rename.ident);
            }
            syn::UseTree::Glob(_) => self.findings.imports.globs.push(prefix.clone()),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.use_tree(prefix, tree);
                }
            }
        }
    }

    fn import(&mut self, name: String, path: Vec<String>, node: &syn::Ident) {
        // An import is a usage too, e.g. `use std::sync::OnceLock;` requires Rust 1.70.
        self.usage(Usage::Path(path.clone()), node);

        // `use x as _` imports a trait without a name
        if name != "_" {
            self.findings.imports.names.insert(name, path);
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        let path = attr.path();

        if path.is_ident("expect") {
            self.feature(LanguageFeature::ExpectLintAttribute, attr);
        } else if path
            .segments
            .first()
            .is_some_and(|s| s.ident == "diagnostic")
        {
            self.feature(LanguageFeature::DiagnosticAttributeNamespace, attr);
        }

        visit::visit_attribute(self, attr);
    }

    fn visit_expr_async(&mut self, node: &'ast syn::ExprAsync) {
        self.feature(LanguageFeature::AsyncAwait, node);
        visit::visit_expr_async(self, node);
    }

    fn visit_expr_await(&mut self, node: &'ast syn::ExprAwait) {
        self.feature(LanguageFeature::AsyncAwait, &node.await_token);
        visit::visit_expr_await(self, node);
    }

    fn visit_expr_block(&mut self, node: &'ast syn::ExprBlock) {
        if let Some(label) = &node.label {
            self.feature(LanguageFeature::LabeledBlock, label);
        }
        visit::visit_expr_block(self, node);
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        if let Some(asyncness) = &node.asyncness {
            self.feature(LanguageFeature::AsyncClosure, asyncness);
        }
        visit::visit_expr_closure(self, node);
    }

    fn visit_expr_const(&mut self, node: &'ast syn::ExprConst) {
        self.feature(LanguageFeature::InlineConst, &node.const_token);
        visit::visit_expr_const(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if is_let_chain(&node.cond) {
            self.feature(LanguageFeature::LetChains, &node.cond);
        }
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        if is_let_chain(&node.cond) {
            self.feature(LanguageFeature::LetChains, &node.cond);
        }
        visit::visit_expr_while(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        self.macro_invocation(&node.mac);
        visit::visit_expr_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.usage(Usage::Method(node.method.to_string()), &node.method);
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none() {
            self.path(&node.path);
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_expr_raw_addr(&mut self, node: &'ast syn::ExprRawAddr) {
        self.feature(LanguageFeature::RawReference, node);
        visit::visit_expr_raw_addr(self, node);
    }

    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry) {
        self.feature(LanguageFeature::QuestionMarkOperator, &node.question_token);
        visit::visit_expr_try(self, node);
    }

    fn visit_generic_param(&mut self, node: &'ast syn::GenericParam) {
        if let syn::GenericParam::Const(param) = node {
            self.feature(LanguageFeature::ConstGenerics, param);
        }
        visit::visit_generic_param(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        self.findings.local_names.functions.insert(name);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_enum(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        self.findings.local_names.functions.insert(name);
        visit::visit_item_fn(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        match &node.ident {
            // A `macro_rules!` definition
            Some(ident) => {
                self.findings.local_names.macros.insert(ident.to_string());
            }
            None => self.macro_invocation(&node.mac),
        }
        visit::visit_item_macro(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_mod(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_struct(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_type(self, node);
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
        self.findings
            .local_names
            .parents
            .insert(node.ident.to_string());
        visit::visit_item_union(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.use_tree(&mut Vec::new(), &node.tree);
        // The paths of the tree are handled by `use_tree`, so the tree is not visited.
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
    }

    fn visit_lit_cstr(&mut self, node: &'ast syn::LitCStr) {
        self.feature(LanguageFeature::CStringLiteral, node);
        visit::visit_lit_cstr(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if node
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
        {
            self.feature(LanguageFeature::LetElse, &node.let_token);
        }
        visit::visit_local(self, node);
    }

    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        if let syn::Pat::Range(range) = node {
            match (&range.limits, &range.end) {
                (syn::RangeLimits::HalfOpen(_), Some(_)) => {
                    self.feature(LanguageFeature::ExclusiveRangePattern, range);
                }
                (syn::RangeLimits::HalfOpen(_), None) => {
                    self.feature(LanguageFeature::HalfOpenRangePattern, range);
                }
                (syn::RangeLimits::Closed(_), _) => {}
            }
        }
        visit::visit_pat(self, node);
    }

    fn visit_signature(&mut self, node: &'ast syn::Signature) {
        if let Some(asyncness) = &node.asyncness {
            self.feature(LanguageFeature::AsyncAwait, asyncness);
        }
        visit::visit_signature(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        self.macro_invocation(&node.mac);
        visit::visit_stmt_macro(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let returns_impl_trait = matches!(
            &node.sig.output,
            syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::ImplTrait(_))
        );

        if node.sig.asyncness.is_some() || returns_impl_trait {
            self.feature(LanguageFeature::AsyncFnAndImplTraitInTrait, &node.sig);
        }

        let name = node.sig.ident.to_string();
        self.findings.local_names.functions.insert(name);
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_trait_item_type(&mut self, node: &'ast syn::TraitItemType) {
        if !node.generics.params.is_empty() {
            self.feature(LanguageFeature::GenericAssociatedTypes, node);
        }
        visit::visit_trait_item_type(self, node);
    }

    fn visit_type_impl_trait(&mut self, node: &'ast syn::TypeImplTrait) {
        self.feature(LanguageFeature::ImplTrait, node);
        visit::visit_type_impl_trait(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.qself.is_none() {
            self.path(&node.path);
        }
        visit::visit_type_path(self, node);
    }

    fn visit_type_trait_object(&mut self, node: &'ast syn::TypeTraitObject) {
        if node.dyn_token.is_some() {
            self.feature(LanguageFeature::DynTrait, node);
        }
        visit::visit_type_trait_object(self, node);
    }
}

fn segments(path: &syn::Path) -> Vec<String> {
    path.segments.iter().map(|s| s.ident.to_string()).collect()
}

fn line(node: &impl Spanned) -> usize {
    node.span().start().line
}

/// Whether a condition chains a `let` with `&&`, e.g. `let Some(x) = y && x > 0`.
fn is_let_chain(cond: &syn::Expr) -> bool {
    fn contains_let(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Let(_) => true,
            syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                contains_let(&binary.left) || contains_let(&binary.right)
            }
            _ => false,
        }
    }

    matches!(cond, syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)))
        && contains_let(cond)
}

/// Whether the format string of a format macro captures an argument, like `{x}`.
///
/// The format string is taken to be the first string literal in the macro input.
/// Named arguments which are given explicitly, like `name = x`, are not captured.
fn has_implicit_format_argument(mac: &syn::Macro) -> bool {
    let tokens: Vec<_> = mac.tokens.clone().into_iter().collect();

    let format_string = tokens.iter().find_map(|tt| match tt {
        proc_macro2::TokenTree::Literal(literal) => {
            syn::parse_str::<syn::LitStr>(&literal.to_string()).ok()
        }
        _ => None,
    });

    let is_explicit = |name: &str| {
        tokens.windows(2).any(|pair| {
            matches!(&pair[0], proc_macro2::TokenTree::Ident(ident) if ident == name)
                && matches!(&pair[1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '=' && p.spacing() == proc_macro2::Spacing::Alone)
        })
    };

    format_string.is_some_and(|s| {
        named_arguments(&s.value())
            .iter()
            .any(|name| !is_explicit(name))
    })
}

/// The names of the arguments in a format string, e.g. `x` for `"{x:?} {0}"`.
fn named_arguments(format_string: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut chars = format_string.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
            }
            '{' => {
                let argument: String = chars
                    .by_ref()
                    .take_while(|&c| c != '}')
                    .collect::<String>()
                    .split(':')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                let is_identifier = argument
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_');

                if is_identifier {
                    names.push(argument);
                }
            }
            _ => {}
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::{has_implicit_format_argument, named_arguments};

    #[yare::parameterized(
        named = { "{x}", &["x"] },
        named_with_spec = { "{x:?}", &["x"] },
        underscore = { "{_x}", &["_x"] },
        positional = { "{}", &[] },
        indexed = { "{0}", &[] },
        indexed_with_spec = { "{0:>8}", &[] },
        escaped = { "{{x}}", &[] },
        escaped_then_named = { "{{}} {x} {y}", &["x", "y"] },
        no_arguments = { "hello", &[] },
    )]
    fn format_string_arguments(format_string: &str, expected: &[&str]) {
        assert_eq!(named_arguments(format_string), expected);
    }

    #[yare::parameterized(
        captured = { r#"println!("{x}")"#, true },
        explicit = { r#"println!("{x}", x = 1)"#, false },
        explicit_and_captured = { r#"println!("{x} {y}", x = 1)"#, true },
        compared_is_not_explicit = { r#"assert!(x == 1, "{x}")"#, true },
        positional = { r#"println!("{}", x)"#, false },
        writer = { r#"write!(f, "{x}")"#, true },
    )]
    fn implicit_format_argument(input: &str, expected: bool) {
        let mac: syn::Macro = syn::parse_str(input).unwrap();

        assert_eq!(has_implicit_format_argument(&mac), expected);
    }
}
