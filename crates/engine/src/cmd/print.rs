//! File › Print: send the slides, notes pages or handouts straight to a printer of the system's
//! spooler (CUPS on macOS and Linux, the Windows print system), with copies, collation, two-sided
//! and greyscale. The pages are the same PDF `file.export` writes, so the Print page's preview,
//! the saved PDF and the paper are one and the same.
//!
//! The web has no spooler: there the PDF downloads and the browser prints it.

#[cfg(not(target_arch = "wasm32"))]
use lamina_model::Presentation;
use serde_json::{Value, json};

#[cfg_attr(target_arch = "wasm32", allow(unused_imports))]
use super::{CommandSpec, EngineError, Result, always, bad, bool_or, cmd, has_doc, str_param};
use crate::Session;

/// The most copies one job asks for.
pub const MAX_COPIES: u32 = 999;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "file.printers",
            "List Printers",
            [],
            None,
            "→ {printers: [{name, description, default, twoSided?}], available: bool} — the system's printers; `available` is false where there is no spooler (the web)",
            always,
            printers
        ),
        cmd!(
            noundo "file.print",
            "Print to Printer",
            ["File"],
            None,
            "{printer?: name (the default printer when omitted), copies?: 1-999, collate?: bool (true), duplex?: off|longEdge|shortEdge, grayscale?: bool, scaling?: exact|shrink, layout?: slides|notes|handouts, perPage?, paper?, orientation?, slides?: [index]|range (\"1-3,5\"), includeHidden?, frame?, dryRun?: bool (check the job and name the command it would run; print nothing)} → {printer, slides, copies, message} or {dryRun: true, command: [..]}",
            has_doc,
            print
        ),
    ]
}

/// One of the system's printers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterInfo {
    pub name: String,
    /// The printer's own description, empty when the spooler gives none.
    pub description: String,
    pub default: bool,
    /// Whether it prints on both sides; `None` when the spooler doesn't say.
    pub two_sided: Option<bool>,
}

impl PrinterInfo {
    /// The name people read: the description when there is one.
    pub fn label(&self) -> &str {
        if self.description.is_empty() { &self.name } else { &self.description }
    }
}

/// Whether this build can reach a spooler at all (the web cannot).
pub const fn available() -> bool {
    cfg!(not(target_arch = "wasm32"))
}

/// The system's printers, the default first. Asks the spooler, which can take a few seconds when
/// it is slow to answer: call it off the window's thread.
pub fn list_printers() -> Vec<PrinterInfo> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut v: Vec<PrinterInfo> = nodalia_print::printers()
            .into_iter()
            .map(|p| PrinterInfo { name: p.name, description: p.description, default: p.default, two_sided: p.two_sided })
            .collect();
        v.sort_by_key(|p| !p.default);
        v
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

fn printers(_s: &mut Session, _p: &Value) -> Result<Value> {
    let list: Vec<Value> = list_printers()
        .into_iter()
        .map(|p| json!({"name": p.name, "description": p.description, "default": p.default, "twoSided": p.two_sided}))
        .collect();
    Ok(json!({"printers": list, "available": available()}))
}

/// The spooler job `p` describes. Checks every option before anything is made.
#[cfg(not(target_arch = "wasm32"))]
pub fn job(p: &Value, title: &str) -> Result<nodalia_print::Job> {
    use nodalia_print::{Duplex, Scaling};
    let copies = match p.get("copies") {
        None | Some(Value::Null) => 1,
        Some(v) => v
            .as_u64()
            .filter(|n| (1..=u64::from(MAX_COPIES)).contains(n))
            .ok_or_else(|| bad("file.print", format!("copies is a number from 1 to {MAX_COPIES}")))? as u32,
    };
    let duplex = match str_param(p, "duplex") {
        None => None,
        Some("off") => Some(Duplex::Off),
        Some("longEdge") => Some(Duplex::LongEdge),
        Some("shortEdge") => Some(Duplex::ShortEdge),
        Some(other) => return Err(bad("file.print", format!("duplex “{other}” isn't off, longEdge or shortEdge"))),
    };
    let scaling = match str_param(p, "scaling") {
        None | Some("exact") => Scaling::Exact,
        Some("shrink") => Scaling::ShrinkToFit,
        Some(other) => return Err(bad("file.print", format!("scaling “{other}” isn't exact or shrink"))),
    };
    let printer = match p.get("printer") {
        None | Some(Value::Null) => None,
        Some(Value::String(name)) if !name.trim().is_empty() && name.len() <= 256 && !name.contains('\0') => Some(name.trim().to_string()),
        Some(_) => return Err(bad("file.print", "printer is the name of one of the system's printers (see file.printers)")),
    };
    Ok(nodalia_print::Job {
        printer,
        copies,
        collate: bool_or(p, "collate", true),
        duplex,
        scaling,
        grayscale: bool_or(p, "grayscale", false),
        // The pages go as they are: a greyscale request is the driver's to honour.
        grey_pages: false,
        title: title.chars().take(120).collect(),
        ..Default::default()
    })
}

/// Print `doc` with the layout and slides `p` names: the PDF, then the spooler. Returns what the
/// spooler said. `dryRun` checks the job and says what would run.
#[cfg(not(target_arch = "wasm32"))]
pub fn print_now(doc: &Presentation, title: &str, p: &Value) -> Result<Value> {
    let job = job(p, title)?;
    let options = super::file::pdf_options(p, doc.slides.len())?;
    let slides = options.slides.as_ref().map_or_else(|| doc.slides.iter().filter(|s| options.include_hidden || !s.hidden).count(), Vec::len);
    if slides == 0 {
        return Err(bad("file.print", "there are no slides to print: every slide in the range is hidden or the range is empty"));
    }
    if bool_or(p, "dryRun", false) {
        let command = nodalia_print::command_preview(&job).map_err(|e| EngineError::Other(e.to_string()))?;
        return Ok(json!({"dryRun": true, "command": command, "slides": slides, "copies": job.copies}));
    }
    let pdf = lamina_pdf::export(doc, &options).map_err(|e| EngineError::Other(e.to_string()))?;
    let message = nodalia_print::submit(&pdf, &job).map_err(|e| EngineError::Other(e.to_string()))?;
    Ok(json!({"printer": job.printer, "slides": slides, "copies": job.copies, "message": message}))
}

fn print(s: &mut Session, p: &Value) -> Result<Value> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (doc, title) = s.doc().map(|d| (d.doc.clone(), d.title()))?;
        print_now(&doc, &title, p)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, p);
        Err(EngineError::Other("printing isn't available on the web: export a PDF with file.export and print it".into()))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn a_job_takes_what_the_params_say() {
        let j = job(
            &json!({"printer": " Office ", "copies": 3, "collate": false, "duplex": "longEdge", "grayscale": true, "scaling": "shrink"}),
            "My deck",
        )
        .expect("job");
        assert_eq!(j.printer.as_deref(), Some("Office"));
        assert_eq!((j.copies, j.collate, j.grayscale), (3, false, true));
        assert_eq!(j.duplex, Some(nodalia_print::Duplex::LongEdge));
        assert_eq!(j.scaling, nodalia_print::Scaling::ShrinkToFit);
        assert_eq!(j.title, "My deck");
    }

    #[test]
    fn a_job_without_params_is_one_collated_copy_to_the_default_printer() {
        let j = job(&json!({}), "d").expect("job");
        assert_eq!((j.printer, j.copies, j.collate, j.duplex, j.grayscale), (None, 1, true, None, false));
    }

    #[test]
    fn bad_options_are_refused_before_anything_is_made() {
        for p in [
            json!({"copies": 0}),
            json!({"copies": 1000}),
            json!({"copies": "two"}),
            json!({"copies": -1}),
            json!({"duplex": "sideways"}),
            json!({"scaling": "zoom"}),
            json!({"printer": 7}),
            json!({"printer": "   "}),
            json!({"printer": "a\0b"}),
        ] {
            assert!(job(&p, "d").is_err(), "{p}");
        }
    }

    #[test]
    fn a_long_title_is_cut() {
        assert_eq!(job(&json!({}), &"x".repeat(500)).expect("job").title.chars().count(), 120);
    }
}
