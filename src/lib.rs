use std::{
    io::{stdin, stdout},
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
use regex::Regex;
use serde::Deserialize;
use tracing::{debug, trace, warn};

pub fn run() -> Result<()> {
    let (ctx, mut book) = CmdPreprocessor::parse_input(stdin())?;
    let raw_config: RawConfig = match ctx.config.get("preprocessor.header-footer") {
        Some(raw) => raw.clone().try_into(),
        None => Ok(Default::default()),
    }?;
    let config = raw_config.compile()?;
    pad_book(&config, &mut book)?;
    serde_json::to_writer(stdout(), &book)?;
    exit(0);
}

pub fn pad_book(config: &Config, book: &mut Book) -> Result<()> {
    book.for_each_mut(|item| {
        if let BookItem::Chapter(Chapter {
            content,
            path: Some(path),
            ..
        }) = item
        {
            match path.to_str() {
                Some(path) => match config.pad_chapter(content, path) {
                    Some(new_content) => *content = new_content,
                    None => debug!(?path, "Did not pad chapter."),
                },
                _ => warn!(?path, "Chapter path is not a valid UTF-8 string."),
            }
        }
    });
    Ok(())
}

pub struct Config {
    headers: Vec<Matcher>,
    footers: Vec<Matcher>,
}

impl Config {
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

pub struct Matcher {
    regex: Regex,
    regex_str: String,
    padding: String,
}

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
