#![cfg_attr(nightly, feature(custom_inner_attributes, proc_macro_hygiene))]

use anyhow::{anyhow, ensure, Result};
use if_chain::if_chain;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use sedregex::find_and_replace;
use std::{
    env,
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
    process::{exit, Command},
};
use syn::{
    parse_file,
    spanned::Spanned,
    visit::{visit_expr_macro, visit_item_macro, visit_stmt_macro, Visit},
    ExprMacro, Ident, ItemMacro, Macro, MacroDelimiter, StmtMacro,
};

mod offset_based_rewriter;

mod offset_calculator;

mod backup;
use backup::Backup;

mod failed_to;
use failed_to::FailedTo;

mod rewriter;
use rewriter::Rewriter;

fn main() -> Result<()> {
    let (args, paths, preformat_failure_is_warning) = process_args();

    if paths.is_empty() {
        return rustfmt(&args, None);
    }

    for path in paths {
        let path = Path::new(&path);

        if let Err(error) = rustfmt(&args, Some(path)) {
            if preformat_failure_is_warning {
                eprintln!("Warning: {error}");
                continue;
            }
            return Err(error);
        }

        let mut backup = Backup::new(path).failed_to(|| format!("backup {path:?}"))?;

        let marker = rewrite_if_chain(path)?;

        rustfmt(&args, Some(path))?;

        restore_if_chain(path, &marker)?;

        backup
            .disable()
            .failed_to(|| format!("disable {path:?} backup"))?;
    }

    Ok(())
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn process_args() -> (Vec<String>, Vec<String>, bool) {
    let mut args = Vec::new();
    let mut paths = Vec::new();
    let mut preformat_failure_is_warning = false;
    for arg in env::args().skip(1) {
        if arg == "--help" || arg == "-h" {
            usage();
        } else if arg == "--preformat-failure-is-warning" {
            preformat_failure_is_warning = true;
        } else if arg.to_lowercase().ends_with(".rs") {
            paths.push(arg);
        } else {
            args.push(arg);
        }
    }
    (args, paths, preformat_failure_is_warning)
}

const USAGE: &str = "\
Usage: rustfmt_if_chain [ARGS]

Arguments ending with `.rs` are considered source files and are
formatted. All other arguments are forwarded to `rustfmt`, with
one exception.

The one argument not forwarded to `rustfmt` is
`--preformat-failure-is-warning`. If this option is passed and
`rustfmt` fails on an unmodified source file, a warning results
instead of an error.\
";

fn usage() -> ! {
    println!("{USAGE}");
    exit(0);
}

fn rewrite_if_chain(path: &Path) -> Result<Ident> {
    let contents = read_to_string(path).failed_to(|| format!("read from {path:?}"))?;

    let marker = unused_ident(&contents);

    let file = parse_file(&contents)
        .map_err(|error| anyhow!("{} at {:?}", error, error.span().start()))
        .failed_to(|| format!("parse {path:?}",))?;

    let mut visitor = RewriteVisitor {
        rewriter: Rewriter::new(&contents),
        marker: &marker,
    };

    visitor.visit_file(&file);

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(path)
        .failed_to(|| format!("open {path:?}"))?;
    file.write_all(visitor.rewriter.contents().as_bytes())
        .failed_to(|| format!("write to {path:?}"))?;

    Ok(marker)
}

fn unused_ident(contents: &str) -> Ident {
    let mut i = 0;
    loop {
        let x = format!("x{i}");
        if !contents.contains(&x) {
            return Ident::new(&x, Span::call_site());
        }
        i += 1;
    }
}

struct RewriteVisitor<'rewrite> {
    rewriter: Rewriter<'rewrite>,
    marker: &'rewrite Ident,
}

impl<'ast, 'rewrite> Visit<'ast> for RewriteVisitor<'rewrite> {
    fn visit_item_macro(&mut self, item_macro: &ItemMacro) {
        if self.rewrite_macro(&item_macro.mac, true) {
            return;
        }
        visit_item_macro(self, item_macro);
    }

    fn visit_stmt_macro(&mut self, stmt_macro: &StmtMacro) {
        if self.rewrite_macro(&stmt_macro.mac, true) {
            return;
        }
        visit_stmt_macro(self, stmt_macro);
    }

    fn visit_expr_macro(&mut self, expr_macro: &ExprMacro) {
        if self.rewrite_macro(&expr_macro.mac, false) {
            return;
        }
        visit_expr_macro(self, expr_macro);
    }
}

impl<'rewrite> RewriteVisitor<'rewrite> {
    fn rewrite_macro(&mut self, mac: &Macro, is_item: bool) -> bool {
        if let Some((span, tokens)) = match_if_chain(mac) {
            let marker = self.marker;
            self.rewrite(
                span,
                &if is_item {
                    quote! { fn #marker() }
                } else {
                    quote! { |#marker| }
                }
                .to_string(),
            );
            self.rewrite_tokens(tokens);
            true
        } else {
            false
        }
    }

    fn rewrite_tokens(&mut self, tokens: &TokenStream) {
        let mut iter = tokens.clone().into_iter().peekable();
        let mut curr_ends_let = if let Some(TokenTree::Ident(ident)) = iter.peek() {
            ident == "let"
        } else {
            false
        };
        while let Some(curr) = iter.next() {
            match (&curr, iter.peek()) {
                (TokenTree::Punct(punct), Some(TokenTree::Ident(next)))
                    if punct.as_char() == ';'
                        && ["if", "let", "then"].contains(&next.to_string().as_str()) =>
                {
                    let marker = self.marker;
                    if !curr_ends_let {
                        self.rewrite(
                            curr.span(),
                            &quote! { { #marker; } }.to_token_stream().to_string(),
                        );
                    }
                    if *next == "then" {
                        self.rewrite(
                            next.span(),
                            &quote! { if #marker }.to_token_stream().to_string(),
                        );
                        return;
                    }
                    curr_ends_let = *next == "let";
                }
                (_, _) => {}
            }
        }
        panic!("`if_chain!` without `then`");
    }

    fn rewrite(&mut self, span: Span, replacement: &str) {
        self.rewriter.rewrite(span, replacement);
    }
}

fn restore_if_chain(path: &Path, marker: &Ident) -> Result<()> {
    let contents = read_to_string(path).failed_to(|| format!("read from {path:?}"))?;

    let contents = find_and_replace(
        &contents,
        &[
            format!(r#"s/(?m)\bfn\s+{marker}\s*\(\)/if_chain!/g"#),
            format!(r#"s/(?m)\|\s*{marker}\s*\|/if_chain!/g"#),
            format!(r#"s/(?m)\s*\{{\s*{marker}\s*;\s*}}/;/g"#),
            format!(r#"s/(?m)\bif\s+{marker}/then/g"#),
        ],
    )?;

    let mut file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(path)
        .failed_to(|| format!("open {path:?}"))?;
    file.write_all(contents.as_bytes())
        .failed_to(|| format!("write to {path:?}"))?;

    Ok(())
}

fn rustfmt(args: &[String], path: Option<&Path>) -> Result<()> {
    let mut command = Command::new("rustfmt");
    command.args(args);
    if let Some(path) = path {
        command.arg(path);
    }
    let status = command
        .status()
        .failed_to(|| format!("get status of {command:?}"))?;

    ensure!(status.success(), "failed to format {:?}", path);

    Ok(())
}

fn match_if_chain(mac: &Macro) -> Option<(Span, &TokenStream)> {
    if_chain! {
        if let Macro {
            path: path @ syn::Path { segments, .. },
            bang_token,
            delimiter: MacroDelimiter::Brace(_),
            tokens,
            ..
        } = mac;
        let segments = segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if let ["if_chain"] = segments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice();
        then {
            Some((
                path.span()
                    .join(bang_token.span())
                    .expect("`path` and `bang_token` should be from the same file"),
                tokens,
            ))
        } else {
            None
        }
    }
}

#[test]
fn usage_wrapping() {
    let re = regex::Regex::new(r"(?m)^.{65,}$").unwrap();
    let unwrapped =
        find_and_replace(USAGE, [r"s/(?P<left>\S)\s(?P<right>\S)/$left $right/g"]).unwrap();
    let mut prev = String::new();
    let mut rewrapped = unwrapped.to_string();
    while re.is_match(&rewrapped) && prev != rewrapped {
        prev = rewrapped;
        rewrapped = find_and_replace(
            &prev,
            [r"s/(?m)^(?P<line>.{0,64})\s/$line
/g"],
        )
        .unwrap()
        .to_string();
    }
    assert_eq!(USAGE, rewrapped);
}

#[test]
fn readme_contains_usage() {
    let readme = read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md")).unwrap();
    assert!(readme.contains(USAGE));
}
