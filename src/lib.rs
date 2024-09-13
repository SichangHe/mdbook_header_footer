use std::{
    io::{stdin, stdout},
    path::PathBuf,
    process::exit,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use derive_everything::derive_everything;
use mdbook_fork4ls::{
    book::{Book, Chapter},
    preprocess::CmdPreprocessor,
    BookItem,
};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use tracing::{debug, trace, warn};

/// Run the header-footer preprocessor: take the book from StdIn, process it,
/// and write it to StdOut.
pub fn run() -> Result<()> {
    let (ctx, mut book) = CmdPreprocessor::parse_input(stdin())?;
    let raw_config: RawConfig = match ctx.config.get("preprocessor.header-footer") {
        Some(raw) => raw.clone().try_into(),
        None => Ok(Default::default()),
    }?;
    let config = raw_config.compile()?;
    config.pad_book(&mut book)?;
    serde_json::to_writer(stdout(), &book)?;
    exit(0);
}

/// Collect all chapter records in the book.
fn all_chapter_records<'a>(items: &'a mut [BookItem], accumulator: &mut Vec<ChapterRecord<'a>>) {
    items.iter_mut().for_each(move |item| {
        if let BookItem::Chapter(Chapter {
            name,
            content,
            path,
            sub_items,
            ..
        }) = item
        {
            accumulator.push(ChapterRecord {
                name,
                content,
                path,
            });
            all_chapter_records(sub_items, accumulator);
        }
    })
}

/// A simplified representation of a chapter for patching.
struct ChapterRecord<'a> {
    name: &'a str,
    content: &'a mut String,
    path: &'a Option<PathBuf>,
}

/// Compiled configuration for the padding.
pub struct Config {
    headers: Vec<Matcher>,
    footers: Vec<Matcher>,
}

impl Config {
    /// Pad the book with headers and footers.
    pub fn pad_book(self, book: &mut Book) -> Result<()> {
        let mut contents_and_paths = Vec::with_capacity(128);
        all_chapter_records(&mut book.sections, &mut contents_and_paths);
        contents_and_paths.into_par_iter().for_each(
            |ChapterRecord {
                 name,
                 content,
                 path,
             }| match path {
                Some(path) => match path.to_str() {
                    Some(path) => match self.pad_chapter(content, path) {
                        Some(new_content) => *content = new_content,
                        None => debug!(?path, "Did not pad chapter."),
                    },
                    _ => warn!(?path, "Chapter path is not a valid UTF-8 string."),
                },
                None => warn!(?name, "Chapter has no path."),
            },
        );
        Ok(())
    }

    /// Pad the chapter with headers and footers.
    pub fn pad_chapter(&self, content: &str, path: &str) -> Option<String> {
        let (mut headers, mut footers) = (Vec::new(), Vec::new());
        for header in &self.headers {
            if header.regex.is_match(path) {
                trace!(?header.regex_str, ?path, "Match");
                headers.push(&header.padding);
            }
        }
        for footer in &self.footers {
            if footer.regex.is_match(path) {
                trace!(?footer.regex_str, ?path, "Match");
                footers.push(&footer.padding);
            }
        }
        if headers.is_empty() && footers.is_empty() {
            None
        } else {
            let capacity = headers.iter().map(|s| s.len()).sum::<usize>()
                + content.len()
                + footers.iter().map(|s| s.len()).sum::<usize>();
            let mut result = String::with_capacity(capacity);
            for header in headers {
                result.push_str(header);
            }
            result.push_str(content);
            for footer in footers {
                result.push_str(footer);
            }
            Some(result)
        }
    }
}

/// A compiled padding instruction.
pub struct Matcher {
    regex: Regex,
    regex_str: String,
    padding: String,
}

/// The raw configuration for padding.
#[derive(Deserialize)]
#[derive_everything]
pub struct RawConfig {
    headers: Vec<RawMatcher>,
    footers: Vec<RawMatcher>,
}

impl RawConfig {
    pub fn compile(self) -> Result<Config> {
        Ok(Config {
            headers: self
                .headers
                .into_iter()
                .map(RawMatcher::compile)
                .collect::<Result<_>>()?,
            footers: self
                .footers
                .into_iter()
                .map(RawMatcher::compile)
                .collect::<Result<_>>()?,
        })
    }
}

/// A raw padding instruction.
#[derive(Deserialize)]
#[derive_everything]
#[serde(rename_all = "kebab-case")]
pub struct RawMatcher {
    #[serde(default = "default_regex_str")]
    regex: String,
    padding: String,
}

impl RawMatcher {
    pub fn compile(self) -> Result<Matcher> {
        Ok(Matcher {
            regex: Regex::new(&self.regex)?,
            regex_str: self.regex,
            padding: self.padding,
        })
    }
}

fn default_regex_str() -> String {
    ".*".into()
}

/// mdBook preprocessor to prepend header and append footer to certain chapters.
#[derive(Parser)]
#[command(version, about)]
pub struct App {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Checks if renderer is supported.
#[derive(Subcommand)]
pub enum Command {
    Supports {
        /// The renderer to check support for.
        renderer: String,
    },
}
