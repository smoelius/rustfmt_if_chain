use anyhow::{anyhow, ensure, Result};
use if_chain::if_chain;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use sedregex::find_and_replace;
use std::{
    env,
    fs::{copy, read_to_string, rename},
    io::Write,
    path::Path,
    process::{exit, Command},
};
use syn::{
    parse_file,
    spanned::Spanned,
    visit::{visit_item, Visit},
    Ident, Item, ItemMacro, Macro,
};
use tempfile::NamedTempFile;

mod rewriter;
use rewriter::Rewriter;

fn main() -> Result<()> {
    let (args, paths, preformat_failure_is_warning) = process_args();

    for path in paths {
        let original = Path::new(&path);

        if let Err(error) = rustfmt(original, &args) {
            if preformat_failure_is_warning {
                eprintln!("Warning: {}", error);
                continue;
            }
            return Err(error);
        }

        let (rewritten, marker) = rewrite_if_chain(original)?;

        let rewritten_formatted = rustfmt(rewritten.path(), &args)?;

        let restored = restore_if_chain(rewritten_formatted.path(), &marker)?;

        rename(restored.path(), original)?;
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

fn usage() -> ! {
    println!(
        "\
Usage: rustfmt_if_chain [ARGS]

Arguments ending with `.rs` are considered source files and are formatted. All other arguments are
forwarded to `rustfmt`, with one exception.

The one argument not forwarded to `rustfmt` is `--preformat-failure-is-warning`. If this option is
passed and `rustfmt` fails on an unmodified source file, a warning results instead of an error.\
"
    );
    exit(0);
}

fn rewrite_if_chain(path: &Path) -> Result<(NamedTempFile, Ident)> {
    let contents = read_to_string(path)?;

    let marker = unused_ident(&contents);

    let file = parse_file(&contents)?;

    let mut visitor = RewriteVisitor {
        rewriter: Rewriter::new(&contents),
        marker: &marker,
    };

    visitor.visit_file(&file);

    let mut tempfile = sibling_tempfile(path)?;
    tempfile
        .as_file_mut()
        .write_all(visitor.rewriter.contents().as_bytes())?;

    Ok((tempfile, marker))
}

fn unused_ident(contents: &str) -> Ident {
    let mut i = 0;
    loop {
        let x = format!("x{}", i);
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
    fn visit_item(&mut self, item: &Item) {
        if let Some((span, tokens)) = match_if_chain(item) {
            let marker = self.marker;
            self.rewrite(span, &quote! { fn #marker() }.to_string());
            self.rewrite_tokens(tokens);
            return;
        }
        visit_item(self, item);
    }
}

impl<'rewrite> RewriteVisitor<'rewrite> {
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

fn restore_if_chain(path: &Path, marker: &Ident) -> Result<NamedTempFile> {
    let contents = read_to_string(path)?;

    let contents = find_and_replace(
        &contents,
        &[
            format!(r#"s/(?m)\bfn\s+{}\s*\(\)/if_chain!/g"#, marker),
            format!(r#"s/(?m)\s*\{{\s*{}\s*;\s*}}/;/g"#, marker),
            format!(r#"s/(?m)\bif\s+{}/then/g"#, marker),
        ],
    )?;

    let mut tempfile = sibling_tempfile(path)?;
    tempfile.as_file_mut().write_all(contents.as_bytes())?;

    Ok(tempfile)
}

fn rustfmt(path: &Path, args: &[String]) -> Result<NamedTempFile> {
    let tempfile = sibling_tempfile(path)?;

    copy(path, tempfile.path())?;

    let mut command = Command::new("rustfmt");
    command.args(args);
    command.arg(tempfile.path());
    let status = command.status()?;

    ensure!(status.success(), "could not format {:?}", path);

    Ok(tempfile)
}

fn sibling_tempfile(path: &Path) -> Result<NamedTempFile> {
    let parent = path.parent().ok_or_else(|| anyhow!("`parent` failed"))?;
    let tempfile = NamedTempFile::new_in(parent)?;
    Ok(tempfile)
}

fn match_if_chain(item: &Item) -> Option<(Span, &TokenStream)> {
    if_chain! {
        if let Item::Macro(ItemMacro {
            mac:
                Macro {
                    path: path @ syn::Path { segments, .. },
                    bang_token,
                    tokens,
                    ..
                },
            ..
        }) = item;
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
