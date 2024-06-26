/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use eipw_lint::fetch::Fetch;
use eipw_lint::lints::{DefaultLint, Lint};
use eipw_lint::modifiers::{DefaultModifier, Modifier};
use eipw_lint::reporters::{AdditionalHelp, Json};
use eipw_lint::{default_lints, Linter, Options};

use js_sys::JsString;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"

export interface Opts {
    allow?: string[];
    warn?: string[];
    deny?: string[];
    default_lints?: Record<string, DefaultLint<string>> | null;
    default_modifiers?: DefaultModifier<string>[] | null;
}

export interface Snippet {
    formatted: string;
}

export interface ProposalRef<S> {
    prefix: S;
    suffix: S;
}

export type LinkFirst<S> = S;

export interface RelativeLinks<S> {
    exceptions: S[];
}

export interface JsonSchema<S> {
    language: S;
    additional_schemas: [S, S][];
    schema: S;
    help: S;
}

export type DefaultModifier<S> = { kind: "set-default-annotation" } & SetDefaultAnnotation<S>;

export interface OneOf<S> {
    name: S;
    values: S[];
}

export type NoDuplicates = null;

export type Date<S> = S;

export type Url<S> = S;

export type Trim = null;

export type Order<S> = S[];

export type UintList<S> = S;

export type Uint<S> = S;

export type Required<S> = S[];

export interface RequiresStatus<S> {
    requires: S;
    status: S;
    flow: S[][];
    prefix: S;
    suffix: S;
}

export interface Length<S> {
    name: S;
    min: number | null;
    max: number | null;
}

export interface Regex<S> {
    mode: Mode;
    pattern: S;
    message: S;
}

export interface RequireReferenced<S> {
    name: S;
    requires: S;
}

export type List<S> = S;

export interface LinkStatus<S> {
    status: S;
    flow: S[][];
    prefix: S;
    suffix: S;
}

export type DefaultLint<S> = { kind: "preamble-author"; name: Author<S> } | { kind: "preamble-date"; name: Date<S> } | ({ kind: "preamble-file-name" } & FileName<S>) | ({ kind: "preamble-length" } & Length<S>) | { kind: "preamble-list"; name: List<S> } | ({ kind: "preamble-no-duplicates" } & NoDuplicates) | ({ kind: "preamble-one-of" } & OneOf<S>) | { kind: "preamble-order"; names: Order<S> } | ({ kind: "preamble-proposal-ref" } & ProposalRef<S>) | ({ kind: "preamble-regex" } & Regex<S>) | ({ kind: "preamble-require-referenced" } & RequireReferenced<S>) | { kind: "preamble-required"; names: Required<S> } | ({ kind: "preamble-required-if-eq" } & RequiredIfEq<S>) | ({ kind: "preamble-requires-status" } & RequiresStatus<S>) | ({ kind: "preamble-trim" } & Trim) | { kind: "preamble-uint"; name: Uint<S> } | { kind: "preamble-uint-list"; name: UintList<S> } | { kind: "preamble-url"; name: Url<S> } | ({ kind: "markdown-html-comments" } & HtmlComments<S>) | ({ kind: "markdown-json-schema" } & JsonSchema<S>) | { kind: "markdown-link-first"; pattern: LinkFirst<S> } | ({ kind: "markdown-link-status" } & LinkStatus<S>) | ({ kind: "markdown-proposal-ref" } & ProposalRef<S>) | ({ kind: "markdown-regex" } & Regex<S>) | ({ kind: "markdown-relative-links" } & RelativeLinks<S>) | { kind: "markdown-section-order"; sections: SectionOrder<S> } | { kind: "markdown-section-required"; sections: SectionRequired<S> } | ({ kind: "markdown-headings-space" } & HeadingsSpace);

export interface HtmlComments<S> {
    name: S;
    warn_for: S[];
}

export interface Regex<S> {
    name: S;
    mode: Mode;
    pattern: S;
    message: S;
}

export type Mode = "includes" | "excludes";

export type SectionRequired<S> = S[];

export interface SetDefaultAnnotation<S> {
    name: S;
    value: S;
    annotation_type: AnnotationTypeDef;
}

export type AnnotationTypeDef = "error" | "warning" | "info" | "note" | "help";

export interface RequiredIfEq<S> {
    when: S;
    equals: S;
    then: S;
}

export interface FileName<S> {
    name: S;
    prefix: S;
    suffix: S;
}

export type HeadingsSpace = null;

export interface ProposalRef<S> {
    name: S;
    prefix: S;
    suffix: S;
}

export type Author<S> = S;

export type SectionOrder<S> = S[];

export function lint(
    sources: string[],
    options?: Opts | null
): Promise<Snippet[]>;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Opts")]
    pub type OptsJS;

    #[wasm_bindgen(typescript_type = "Snippet")]
    pub type SnippetJS;
}

#[derive(Debug)]
struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

#[wasm_bindgen(module = "node:fs/promises")]
extern "C" {
    #[wasm_bindgen(catch, js_name = readFile)]
    async fn read_file(path: &JsString, encoding: &JsString) -> Result<JsValue, JsValue>;
}

struct NodeFetch;

impl Fetch for NodeFetch {
    fn fetch(
        &self,
        path: PathBuf,
    ) -> Pin<Box<dyn Future<Output = Result<String, std::io::Error>>>> {
        let fut = async move {
            let path = match path.to_str() {
                Some(p) => JsString::from(p),
                None => return Err(std::io::ErrorKind::InvalidInput.into()),
            };

            let encoding = JsString::from("utf-8");

            match read_file(&path, &encoding).await {
                Ok(o) => Ok(o.as_string().unwrap()),
                Err(e) => {
                    let txt = format!("{:?}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error(txt)))
                }
            }
        };

        Box::pin(fut)
    }
}

#[derive(Debug, Deserialize)]
struct Opts {
    #[serde(default)]
    allow: Vec<String>,

    #[serde(default)]
    warn: Vec<String>,

    #[serde(default)]
    deny: Vec<String>,

    #[serde(default)]
    default_lints: Option<HashMap<String, DefaultLint<String>>>,

    #[serde(default)]
    default_modifiers: Option<Vec<DefaultModifier<String>>>,
}

impl Opts {
    fn apply<'a, 'b: 'a, R>(&'a self, mut linter: Linter<'b, R>) -> Linter<'a, R> {
        for allow in &self.allow {
            linter = linter.allow(allow);
        }

        if !self.warn.is_empty() {
            let mut lints: HashMap<_, _> = default_lints().collect();
            for warn in &self.warn {
                let (k, v) = lints.remove_entry(warn.as_str()).unwrap();
                linter = linter.warn(k, v);
            }
        }

        if !self.deny.is_empty() {
            let mut lints: HashMap<_, _> = default_lints().collect();
            for deny in &self.deny {
                let (k, v) = lints.remove_entry(deny.as_str()).unwrap();
                linter = linter.deny(k, v);
            }
        }

        linter
    }
}

#[wasm_bindgen]
pub async fn lint(sources: Vec<JsString>, options: Option<OptsJS>) -> Result<JsValue, JsError> {
    let sources: Vec<_> = sources
        .into_iter()
        .map(|v| v.as_string().unwrap())
        .map(PathBuf::from)
        .collect();

    let reporter = Json::default();
    let reporter = AdditionalHelp::new(reporter, |t: &str| {
        Ok(format!("see https://ethereum.github.io/eipw/{}/", t))
    });

    let opts: Opts;
    let mut linter;
    if let Some(options) = options {
        opts = serde_wasm_bindgen::from_value(options.deref().clone())?;

        let mut options = Options::default();

        if let Some(ref lints) = opts.default_lints {
            options.lints = Some(
                lints
                    .iter()
                    .map(|(k, v)| (k.as_str(), Box::new(v.clone()) as Box<dyn Lint>)),
            );
        }

        if let Some(ref modifiers) = opts.default_modifiers {
            options.modifiers = Some(
                modifiers
                    .iter()
                    .map(|m| Box::new(m.clone()) as Box<dyn Modifier>),
            );
        }

        linter = Linter::with_options(reporter, options);
        linter = opts.apply(linter);
    } else {
        linter = Linter::new(reporter);
    }

    linter = linter.set_fetch(NodeFetch);

    for source in &sources {
        linter = linter.check_file(source);
    }

    let reporter = linter.run().await?;

    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    let js_value = reporter
        .into_inner()
        .into_reports()
        .serialize(&serializer)
        .unwrap();

    Ok(js_value)
}

#[wasm_bindgen]
pub fn format(snippet: &SnippetJS) -> Result<String, JsError> {
    let value: serde_json::Value = serde_wasm_bindgen::from_value(snippet.deref().clone())?;

    let obj = match value {
        serde_json::Value::Object(o) => o,
        _ => return Err(JsError::new("expected object")),
    };

    match obj.get("formatted") {
        Some(serde_json::Value::String(s)) => Ok(s.into()),
        _ => Err(JsError::new("expected `formatted` to be a string")),
    }
}
