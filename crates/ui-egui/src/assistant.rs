//! The AI assistant: Lámina's side of the Office suite's shared chat (`nodalia-assistant`, from
//! nodalia-design `packages/assistant`).
//!
//! The component draws the button, the panel and the Preferences section the same way in Letra,
//! Celdas and here, keeps the person's provider, model and key, and runs the loop with the model.
//! This file is everything the component cannot know: what a presentation is, which tools the
//! model gets to work on it, and how a turn is undone. The tools sit on the same command registry
//! that scripts and MCP use, so the assistant can do what a person can do from the ribbon and
//! nothing else: it cannot open, save, close, export or print files, bring in a file from the
//! disk, or start the slide show.
//!
//! The person's own messages are the only instructions. The text on the slides is data, and every
//! change waits for Apply unless they chose to let the assistant work as it goes (Preferences). A
//! whole turn undoes in one click.

use std::sync::Arc;

use egui::Ui;
use nodalia_assistant::{Assistant, Host, Lang, ToolSpec};
use nodalia_theme::office::Look;
use serde_json::{Value, json};

use crate::SlideApp;
use crate::theme::{self, Tokens};

/// Slides the snapshot lists; the model reads any slide with `read_slide`.
const MAX_SLIDES: usize = 200;
/// Characters of a slide's notes the snapshot keeps.
const NOTES_PREVIEW: usize = 160;
/// Most commands one answer lists.
const MAX_LISTED: usize = 60;
/// Longest text the approval card shows for one argument.
const PREVIEW: usize = 80;
/// The largest table the assistant builds in one call.
const MAX_ROWS: usize = 60;
const MAX_COLS: usize = 20;

/// Command families the assistant never runs: files, windows and the slide show are the person's
/// to open, save, print and start; `view.*` moves the window rather than the deck; a command that
/// reads a file from the disk would bring in whatever the text of a slide asks for; and Copy and
/// Cut write the system clipboard.
const FORBIDDEN: [&str; 14] = [
    "file.",
    "ui.",
    "app.",
    "window.",
    "print",
    "show.",
    "view.",
    "media.",
    "insert.picture",
    "insert.audio",
    "insert.video",
    "insert.photoAlbum",
    "review.compare",
    "edit.c",
];

/// The look the shared component draws with: Lámina's palette and its bold family.
fn look(t: &Tokens) -> Look {
    Look::new(theme::palette(t.dark)).with_bold(egui::FontFamily::Name("bold".into()))
}

/// The language the component speaks: Spanish when the interface is Spanish, English otherwise.
fn lang() -> Lang {
    match crate::i18n::current() {
        crate::i18n::Language::Es => Lang::Spanish,
        crate::i18n::Language::En => Lang::English,
    }
}

impl SlideApp {
    /// The assistant, made the first time it is asked for. It reads the saved choices and starts
    /// asking the keychain, which nothing should do before a window is on screen.
    pub(crate) fn assistant_mut(&mut self, ctx: &egui::Context) -> &mut Assistant {
        let ctx = ctx.clone();
        self.assistant.get_or_insert_with(|| Assistant::new(Arc::new(move || ctx.request_repaint())))
    }

    /// Runs `f` with the assistant and a host over the active presentation. The assistant lives
    /// in the app and the host borrows the app, so it is taken out for the duration.
    fn with_assistant<R>(&mut self, ctx: &egui::Context, f: impl FnOnce(&mut Assistant, &mut LaminaHost<'_>) -> R) -> R {
        self.assistant_mut(ctx);
        let lang = lang();
        let mut assistant = self.assistant.take().unwrap_or_else(|| Assistant::new(Arc::new(|| {})));
        assistant.lang = lang;
        let result = f(&mut assistant, &mut LaminaHost { app: self, lang });
        self.assistant = Some(assistant);
        result
    }

    /// Opens or closes the assistant's chat (`ui.assistant`, and the snapshot example). `None`
    /// switches it. Returns whether it is open.
    pub fn show_assistant(&mut self, ctx: &egui::Context, open: Option<bool>) -> bool {
        let a = self.assistant_mut(ctx);
        a.open = open.unwrap_or(!a.open);
        a.open
    }

    /// Runs the model's pending tool calls and moves its answer along. Called once a frame.
    pub(crate) fn poll_assistant(&mut self, ctx: &egui::Context) {
        if self.assistant.is_none() {
            return;
        }
        if self.with_assistant(ctx, |a, host| a.poll(host)) {
            ctx.request_repaint();
        }
    }
}

/// Whether the title bar offers the assistant: the person did not turn it off.
pub(crate) fn offered(app: &mut SlideApp, ctx: &egui::Context) -> bool {
    app.assistant_mut(ctx).settings.enabled
}

/// The title bar's button: the chosen provider's mark, which opens and closes the chat.
pub(crate) fn button(app: &mut SlideApp, ui: &mut Ui) {
    let look = look(&Tokens::get(ui.ctx()));
    let lang = lang();
    let a = app.assistant_mut(ui.ctx());
    a.lang = lang;
    let label = lang.t("Assistant");
    let r = nodalia_assistant::ui::button(ui, &look, a);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
}

/// The chat panel, on the right edge, drawn before the other panes so it keeps the edge.
pub(crate) fn panel(app: &mut SlideApp, ui: &mut Ui) {
    if !app.assistant.as_ref().is_some_and(|a| a.open) {
        return;
    }
    let ctx = ui.ctx().clone();
    let look = look(&Tokens::get(&ctx));
    let gap = theme::ISLAND_GAP;
    let mut open_preferences = false;
    egui::Panel::right("assistant_panel")
        .exact_size(nodalia_assistant::ui::PANEL_WIDTH + gap)
        .show_separator_line(false)
        .frame(egui::Frame::NONE.inner_margin(egui::Margin { left: gap as i8, ..Default::default() }))
        .show(ui, |ui| {
            open_preferences = app.with_assistant(&ctx, |a, host| nodalia_assistant::ui::panel(ui, &look, a, host)).open_preferences;
        });
    if open_preferences {
        app.open_backstage(crate::backstage::Page::Options);
    }
}

/// The Preferences section.
pub(crate) fn preferences(app: &mut SlideApp, ui: &mut Ui) {
    let look = look(&Tokens::get(ui.ctx()));
    let lang = lang();
    let a = app.assistant_mut(ui.ctx());
    a.lang = lang;
    nodalia_assistant::ui::preferences(ui, &look, a);
}

/// What the assistant works through: the active presentation, by the commands the registry has.
pub(crate) struct LaminaHost<'a> {
    app: &'a mut SlideApp,
    lang: Lang,
}

fn tool(name: &str, description: &str, schema: Value, read_only: bool) -> ToolSpec {
    ToolSpec { name: name.into(), description: description.into(), schema, read_only }
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type": "object", "properties": properties, "required": required})
}

fn text(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

fn flag(description: &str) -> Value {
    json!({"type": "boolean", "description": description})
}

fn number(description: &str) -> Value {
    json!({"type": "number", "description": description})
}

fn ids_schema(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "integer"}, "description": description})
}

fn forbidden(command: &str) -> bool {
    FORBIDDEN.iter().any(|f| command.starts_with(f))
}

/// The first `n` characters of `s`, with an ellipsis when it was longer.
fn clip(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().nth(n).is_some() {
        out.push('…');
    }
    out
}

fn str_arg<'v>(input: &'v Value, key: &str) -> Result<&'v str, String> {
    input.get(key).and_then(Value::as_str).ok_or_else(|| format!("`{key}` is missing; it must be a string"))
}

fn id_arg(input: &Value, key: &str) -> Result<i64, String> {
    input.get(key).and_then(Value::as_i64).ok_or_else(|| format!("`{key}` is missing; it must be a shape id from read_slide"))
}

/// The shape ids the model gave: `ids`, or the single `id`.
fn ids_arg(input: &Value) -> Result<Vec<i64>, String> {
    let mut ids: Vec<i64> = input.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
    if let Some(id) = input.get("id").and_then(Value::as_i64) {
        ids.push(id);
    }
    if ids.is_empty() { Err("give `ids` (or `id`): shape ids from read_slide".into()) } else { Ok(ids) }
}

/// Copies the listed keys (when present and not null) into a params object.
fn pick(input: &Value, keys: &[&str]) -> Value {
    Value::Object(keys.iter().filter_map(|k| input.get(*k).filter(|v| !v.is_null()).map(|v| (k.to_string(), v.clone()))).collect())
}

/// The steps that format the text of the selected shapes: one command per property given.
fn text_steps(input: &Value) -> Result<Vec<(&'static str, Value)>, String> {
    let mut steps: Vec<(&'static str, Value)> = Vec::new();
    for (key, id) in [("bold", "format.bold"), ("italic", "format.italic"), ("shadow", "format.textShadow")] {
        if let Some(on) = input.get(key).and_then(Value::as_bool) {
            steps.push((id, json!({"on": on})));
        }
    }
    if let Some(v) = input.get("font").and_then(Value::as_str) {
        steps.push(("format.font", json!({"family": v})));
    }
    if let Some(v) = input.get("size").and_then(Value::as_f64) {
        steps.push(("format.size", json!({"size": v})));
    }
    if let Some(v) = input.get("color").and_then(Value::as_str) {
        steps.push(("format.color", json!({"color": v})));
    }
    if let Some(v) = input.get("align").and_then(Value::as_str) {
        let id = match v {
            "left" => "format.alignLeft",
            "center" => "format.alignCenter",
            "right" => "format.alignRight",
            "justify" => "format.justify",
            other => return Err(format!("align `{other}` must be left, center, right or justify")),
        };
        steps.push((id, json!({})));
    }
    if let Some(v) = input.get("anchor").and_then(Value::as_str) {
        steps.push(("format.anchor", json!({"anchor": v})));
    }
    if let Some(on) = input.get("bullets").and_then(Value::as_bool) {
        steps.push(("format.bullets", json!({"on": on})));
    }
    if let Some(v) = input.get("lineSpacing").and_then(Value::as_f64) {
        steps.push(("format.lineSpacing", json!({"lines": v})));
    }
    if steps.is_empty() {
        return Err("nothing to apply: give at least one of bold, italic, font, size, color, align, anchor, bullets, lineSpacing, shadow".into());
    }
    Ok(steps)
}

/// The steps that style shapes: fill and outline, on the shapes in `ids`.
fn shape_steps(input: &Value, ids: &[i64]) -> Vec<(&'static str, Value)> {
    let mut steps: Vec<(&'static str, Value)> = Vec::new();
    match input.get("fill") {
        Some(Value::String(c)) if c == "none" => steps.push(("shape.fill", json!({"none": true, "ids": ids}))),
        Some(Value::String(c)) => steps.push(("shape.fill", json!({"color": c, "ids": ids}))),
        Some(Value::Object(g)) => steps.push(("shape.fill", json!({"gradient": g, "ids": ids}))),
        _ => {}
    }
    if let Some(a) = input.get("transparency").and_then(Value::as_f64) {
        steps.push(("shape.fill", json!({"transparency": a, "ids": ids})));
    }
    match input.get("outline") {
        Some(Value::String(c)) if c == "none" => steps.push(("shape.line", json!({"none": true, "ids": ids}))),
        Some(Value::String(c)) => steps.push(("shape.line", json!({"color": c, "ids": ids}))),
        Some(Value::Object(o)) => {
            let mut params = o.clone();
            params.insert("ids".into(), json!(ids));
            steps.push(("shape.line", Value::Object(params)));
        }
        _ => {}
    }
    if let Some(v) = input.get("shadow").and_then(Value::as_str) {
        steps.push(("shape.effects", json!({"shadow": v, "ids": ids})));
    }
    steps
}

/// A slide as the model reads it: the shapes and what is on them, without the animation tables
/// and the formatting of the notes, which `run_command` reads when needed.
fn trim_slide(mut slide: Value) -> Value {
    if let Some(o) = slide.as_object_mut() {
        o.remove("notesBody");
        if let Some(n) = o.get("animations").and_then(Value::as_array).map(Vec::len) {
            o.insert("animations".into(), json!(n));
        }
    }
    slide
}

impl LaminaHost<'_> {
    fn run(&mut self, command: &str, params: Value) -> Result<Value, String> {
        self.app.run(command, params)
    }

    /// Text the person is still typing into a shape belongs to the slide before the assistant
    /// changes anything, so the draft is entered first and never lost or overwritten.
    fn edit(&mut self, command: &str, params: Value) -> Result<Value, String> {
        let _ = self.app.session.execute("text.exit", &json!({}));
        self.run(command, params)
    }

    /// Runs the steps in order on the shapes `ids`, selecting them first. A failure says which
    /// step failed and what had been applied.
    fn steps(&mut self, ids: &[i64], steps: Vec<(&'static str, Value)>) -> Result<Vec<&'static str>, String> {
        self.edit("edit.select", json!({"ids": ids}))?;
        let mut applied: Vec<&'static str> = Vec::new();
        for (id, params) in steps {
            self.run(id, params).map_err(|e| format!("{id}: {e} (applied before the failure: {applied:?})"))?;
            applied.push(id);
        }
        Ok(applied)
    }

    fn list_commands(&mut self, query: &str) -> Value {
        let q = query.to_lowercase();
        let found: Vec<Value> = self
            .app
            .session
            .commands()
            .into_iter()
            .filter(|c| !forbidden(c.id))
            .filter(|c| {
                q.is_empty()
                    || c.id.to_lowercase().contains(&q)
                    || c.label.to_lowercase().contains(&q)
                    || c.menu.iter().any(|m| m.to_lowercase().contains(&q))
            })
            .take(MAX_LISTED)
            .map(|c| json!({"id": c.id, "label": c.label, "where": c.menu.join(" › "), "params": c.params, "enabled": c.enabled}))
            .collect();
        json!({"commands": found, "limit": MAX_LISTED})
    }

    /// The deck as the snapshot shows it: every slide's title and layout, and the slide the
    /// person is on in full.
    fn deck(&mut self) -> Result<Value, String> {
        let mut deck = self.run("document.inspect", json!({}))?;
        if let Some(o) = deck.as_object_mut() {
            // The person's own undo labels say nothing about the deck.
            o.remove("undo");
            o.remove("redo");
            if let Some(slides) = o.get_mut("slides").and_then(Value::as_array_mut) {
                let total = slides.len();
                slides.truncate(MAX_SLIDES);
                for s in slides.iter_mut() {
                    if let Some(n) = s.get("notes").and_then(Value::as_str).map(|n| clip(n, NOTES_PREVIEW)) {
                        s["notes"] = json!(n);
                    }
                }
                if total > MAX_SLIDES {
                    o.insert(
                        "slidesNote".into(),
                        json!(format!("Only the first {MAX_SLIDES} of {total} slides are listed. Read the others with read_slide.")),
                    );
                }
            }
            if let Ok(current) = self.run("slide.inspect", json!({})) {
                o.insert("currentSlide".into(), trim_slide(current));
            }
        }
        Ok(deck)
    }
}

const GUIDE: &str = "Lámina is a presentation app. A presentation is a list of slides (numbered from 0), each laid out from a layout of the theme, and a slide holds shapes: placeholders (title, body), text boxes, shapes, pictures, tables, charts and diagrams, each with an id. Positions and sizes are in points on a slide that is usually 960 × 540, as [x, y, width, height] from the top-left corner.\n\
\n\
The snapshot gives the deck (every slide's title, layout and notes), the theme, the selection and the slide the person is on in full. Where the person is working is that slide. Read any other slide with read_slide; shape ids come from there. New shapes land on the current slide, so add_slide and go_to_slide say where the next ones go.\n\
\n\
Build slides from layouts rather than loose boxes: add_slide with a layout (title, titleAndContent, sectionHeader, twoContent, comparison, titleOnly, blank, contentWithCaption, pictureWithCaption) and a title and body, which fills the placeholders so the slide follows the theme. Put short text on slides: a headline of a few words and at most five or six lines, the detail in the speaker notes (set_notes). Change a theme's colours and fonts with run_command design.theme rather than restyling every shape, and keep to the theme's colours (accent1…accent6, tx1, bg1) instead of fixed ones when you can.\n\
\n\
For data use add_chart (categories and series), for a process, cycle or hierarchy add_diagram, for a grid add_table. Check the result with read_slide, and run check_accessibility before you say a deck is done.\n\
\n\
Lámina has hundreds of commands, one for each ribbon button. Look one up with list_commands (search by what it does, like \"animation\", \"transition\", \"align\") and read its params before running it with run_command. You cannot open, save, close, export or print files, bring in a picture from the disk, or start the slide show.";

impl Host for LaminaHost<'_> {
    fn app_name(&self) -> &str {
        "Lámina"
    }

    fn guide(&self) -> String {
        GUIDE.into()
    }

    fn context(&mut self) -> Result<Value, String> {
        self.deck()
    }

    fn tools(&self) -> Vec<ToolSpec> {
        let rect = "[x, y, width, height] in points from the slide's top-left corner";
        let colour = "A colour: \"#RRGGBB\", or a theme colour such as accent1…accent6, tx1, tx2, bg1, bg2";
        vec![
            tool(
                "read_slide",
                "Read a slide: its layout, background, notes, transition, animations (a count) and every shape with its id, kind, box [x, y, w, h], fill, rotation and text. Without `index` reads the current slide.",
                object(json!({"index": number("Slide number from 0 (default: the current slide)")}), &[]),
                true,
            ),
            tool(
                "read_shape",
                "Everything about one shape: its full formatting and its effective box.",
                object(json!({"id": number("A shape id from read_slide")}), &["id"]),
                true,
            ),
            tool(
                "go_to_slide",
                "Make a slide the current one, which is where new shapes are added. Does not change the deck.",
                object(json!({"index": number("Slide number from 0")}), &["index"]),
                true,
            ),
            tool(
                "check_accessibility",
                "List accessibility problems in the deck: pictures without alternative text, empty titles, low contrast and the like, as {slide, shape, issue}.",
                object(json!({}), &[]),
                true,
            ),
            tool(
                "list_commands",
                "Find Lámina commands by what they do. Searches each command's id, label and ribbon path and says whether it can run now. Returns up to 60.",
                object(json!({"query": text("Words to look for, e.g. \"transition\", \"animation\", \"align\"")}), &[]),
                true,
            ),
            tool(
                "add_slide",
                "Add a slide from a layout and make it the current one. `title` and `body` fill the layout's placeholders (a new line starts a bullet in the body). Returns the slide's id and index.",
                object(
                    json!({
                        "layout": {"type": "string", "enum": ["title", "titleAndContent", "sectionHeader", "twoContent", "comparison", "titleOnly", "blank", "contentWithCaption", "pictureWithCaption"]},
                        "title": text("The slide's title"),
                        "body": text("The body text, one bullet per line"),
                        "at": number("Where to put it, as a slide number from 0 (default: after the current slide)"),
                    }),
                    &[],
                ),
                false,
            ),
            tool(
                "set_text",
                "Replace all the text of a shape (or of one table cell with `cell`), keeping the first run's formatting. A new line starts a new paragraph.",
                object(
                    json!({"id": number("A shape id from read_slide"), "text": text("The new text"), "cell": {"type": "array", "items": {"type": "integer"}, "description": "[row, column] of a table cell, from 0"}}),
                    &["id", "text"],
                ),
                false,
            ),
            tool(
                "add_text_box",
                "Add a text box to the current slide. Returns its id.",
                object(json!({"text": text("The text"), "rect": {"type": "array", "items": {"type": "number"}, "description": rect}}), &["text"]),
                false,
            ),
            tool(
                "add_shape",
                "Add a shape to the current slide, with optional text, fill and outline. Returns its id.",
                object(
                    json!({
                        "preset": text("rect, roundRect, ellipse, triangle, rightArrow, star5, … (list them with run_command shape.presets)"),
                        "rect": {"type": "array", "items": {"type": "number"}, "description": rect},
                        "text": text("Text inside the shape"),
                        "fill": text(colour),
                        "outline": text(colour),
                    }),
                    &["preset", "rect"],
                ),
                false,
            ),
            tool(
                "style_shapes",
                "Fill and outline shapes. `fill` and `outline` take a colour or \"none\"; `outline` can also be {color, width, dash}; `fill` can be a gradient {stops: [[0..1, colour]], kind, angle}; `shadow` is none, outer, inner or perspective.",
                object(
                    json!({
                        "ids": ids_schema("Shape ids from read_slide"),
                        "fill": {"description": colour},
                        "transparency": number("Fill transparency from 0 (opaque) to 1"),
                        "outline": {"description": "A colour, \"none\", or {color, width, dash}"},
                        "shadow": {"type": "string", "enum": ["none", "outer", "inner", "perspective"]},
                    }),
                    &["ids"],
                ),
                false,
            ),
            tool(
                "format_text",
                "Format all the text of shapes: bold, italic, font, size, colour, alignment, vertical anchor, bullets, line spacing.",
                object(
                    json!({
                        "ids": ids_schema("Shape ids from read_slide"),
                        "bold": flag("Bold on or off"),
                        "italic": flag("Italic on or off"),
                        "shadow": flag("Text shadow on or off"),
                        "font": text("Font family"),
                        "size": json!({"type": "number", "minimum": 1, "maximum": 400, "description": "Size in points"}),
                        "color": text(colour),
                        "align": {"type": "string", "enum": ["left", "center", "right", "justify"]},
                        "anchor": {"type": "string", "enum": ["top", "middle", "bottom"]},
                        "bullets": flag("Bullets on or off"),
                        "lineSpacing": number("Line spacing in lines, e.g. 1.0 or 1.5"),
                    }),
                    &["ids"],
                ),
                false,
            ),
            tool(
                "move_shapes",
                "Move, resize or rotate shapes. `x` and `y` set the top-left corner, `w` and `h` the size, `rotation` the angle in degrees; give only what changes.",
                object(
                    json!({
                        "ids": ids_schema("Shape ids from read_slide"),
                        "x": number("Left edge in points"),
                        "y": number("Top edge in points"),
                        "w": number("Width in points"),
                        "h": number("Height in points"),
                        "rotation": number("Angle in degrees"),
                    }),
                    &["ids"],
                ),
                false,
            ),
            tool(
                "delete_shapes",
                "Delete shapes from the current slide.",
                object(json!({"ids": ids_schema("Shape ids from read_slide")}), &["ids"]),
                false,
            ),
            tool(
                "add_table",
                "Add a table to the current slide, optionally filled from `cells` (rows of text, the first row usually the header). Returns its id.",
                object(
                    json!({
                        "rows": number("Number of rows (at most 60)"),
                        "cols": number("Number of columns (at most 20)"),
                        "cells": {"type": "array", "description": "Rows of cell text", "items": {"type": "array", "items": {"type": ["string", "number", "boolean", "null"]}}},
                        "rect": {"type": "array", "items": {"type": "number"}, "description": rect},
                        "style": text("A table style, e.g. \"medium2-accent1\", \"light1-accent2\", \"grid\""),
                    }),
                    &["rows", "cols"],
                ),
                false,
            ),
            tool(
                "add_chart",
                "Add a chart to the current slide from categories and series. Returns its id.",
                object(
                    json!({
                        "type": {"type": "string", "enum": ["column", "bar", "line", "pie", "doughnut", "area", "scatter", "stackedColumn", "radar"]},
                        "categories": {"type": "array", "items": {"type": ["string", "number"]}, "description": "The category labels"},
                        "series": {"type": "array", "description": "One entry per series", "items": {"type": "object", "properties": {"name": {"type": "string"}, "values": {"type": "array", "items": {"type": "number"}}}, "required": ["name", "values"]}},
                        "title": text("Chart title"),
                        "legend": {"type": "string", "enum": ["b", "t", "l", "r"], "description": "Where the legend goes (omit for none)"},
                        "dataLabels": flag("Show the values on the chart"),
                        "rect": {"type": "array", "items": {"type": "number"}, "description": rect},
                    }),
                    &["type", "categories", "series"],
                ),
                false,
            ),
            tool(
                "add_diagram",
                "Add a diagram of text items to the current slide: a list, a process, a cycle, a hierarchy, a pyramid or a matrix. A leading tab on an item makes it a sub-point. Returns its id.",
                object(
                    json!({
                        "kind": {"type": "string", "enum": ["list", "process", "cycle", "hierarchy", "pyramid", "matrix"]},
                        "items": {"type": "array", "items": {"type": "string"}, "description": "The text of each node"},
                    }),
                    &["kind", "items"],
                ),
                false,
            ),
            tool(
                "set_notes",
                "Set a slide's speaker notes (replacing them).",
                object(json!({"text": text("The notes"), "index": number("Slide number from 0 (default: the current slide)")}), &["text"]),
                false,
            ),
            tool(
                "set_transition",
                "Set the transition into a slide: none, fade, push, wipe, split, reveal, cut, morph, cover, uncover, flash… (list them with run_command transition.list). `all` applies it to every slide.",
                object(
                    json!({
                        "kind": text("Transition name"),
                        "option": text("Effect option, e.g. a direction"),
                        "duration": number("Length in milliseconds"),
                        "index": number("Slide number from 0 (default: the current slide)"),
                        "all": flag("Apply to every slide"),
                    }),
                    &["kind"],
                ),
                false,
            ),
            tool(
                "run_command",
                "Run a Lámina command by id with params, as the ribbon button would on the selection, e.g. animation.add, slide.layout, design.theme, arrange.align or slide.duplicate. Find ids with list_commands. File, window, slide show and import-a-file commands are not available.",
                object(json!({"command": text("The command id"), "params": {"type": "object", "description": "The command's params"}}), &["command"]),
                false,
            ),
        ]
    }

    fn call(&mut self, tool: &str, input: &Value) -> Result<Value, String> {
        match tool {
            "read_slide" => self.run("slide.inspect", pick(input, &["index"])).map(trim_slide),
            "read_shape" => self.run("shape.inspect", pick(input, &["id"])),
            "go_to_slide" => self.run("slide.go", pick(input, &["index"])),
            "check_accessibility" => self.run("review.accessibility", json!({})),
            "list_commands" => Ok(self.list_commands(input.get("query").and_then(Value::as_str).unwrap_or(""))),
            "add_slide" => self.edit("slide.new", pick(input, &["layout", "title", "body", "at"])),
            "set_text" => {
                id_arg(input, "id")?;
                str_arg(input, "text")?;
                self.edit("text.set", pick(input, &["id", "text", "cell"]))
            }
            "add_text_box" => {
                str_arg(input, "text")?;
                self.edit("insert.textBox", pick(input, &["text", "rect"]))
            }
            "add_shape" => {
                let made = self.edit("shape.insert", pick(input, &["preset", "rect", "text"]))?;
                let id = made.get("id").and_then(Value::as_i64).ok_or("the shape was added but no id came back")?;
                let steps = shape_steps(input, &[id]);
                if !steps.is_empty() {
                    self.steps(&[id], steps)?;
                }
                Ok(made)
            }
            "style_shapes" => {
                let ids = ids_arg(input)?;
                let steps = shape_steps(input, &ids);
                if steps.is_empty() {
                    return Err("nothing to apply: give at least one of fill, transparency, outline, shadow".into());
                }
                self.edit("text.exit", json!({}))?;
                let applied = self.steps(&ids, steps)?;
                Ok(json!({"ids": ids, "applied": applied}))
            }
            "format_text" => {
                let ids = ids_arg(input)?;
                let steps = text_steps(input)?;
                self.edit("text.exit", json!({}))?;
                let applied = self.steps(&ids, steps)?;
                Ok(json!({"ids": ids, "applied": applied}))
            }
            "move_shapes" => {
                let ids = ids_arg(input)?;
                let mut steps: Vec<(&'static str, Value)> = Vec::new();
                if input.get("x").is_some() || input.get("y").is_some() {
                    let mut p = pick(input, &["x", "y"]);
                    p["ids"] = json!(ids);
                    steps.push(("shape.move", p));
                }
                if input.get("w").is_some() || input.get("h").is_some() {
                    let mut p = pick(input, &["w", "h"]);
                    p["ids"] = json!(ids);
                    steps.push(("shape.resize", p));
                }
                if let Some(deg) = input.get("rotation").and_then(Value::as_f64) {
                    steps.push(("shape.rotate", json!({"deg": deg, "ids": ids})));
                }
                if steps.is_empty() {
                    return Err("nothing to change: give at least one of x, y, w, h, rotation".into());
                }
                self.edit("text.exit", json!({}))?;
                let applied = self.steps(&ids, steps)?;
                Ok(json!({"ids": ids, "applied": applied}))
            }
            "delete_shapes" => {
                let ids = ids_arg(input)?;
                self.edit("edit.delete", json!({"ids": ids}))
            }
            "add_table" => {
                let rows = input.get("rows").and_then(Value::as_u64).ok_or("`rows` must be a number")? as usize;
                let cols = input.get("cols").and_then(Value::as_u64).ok_or("`cols` must be a number")? as usize;
                if rows == 0 || cols == 0 || rows > MAX_ROWS || cols > MAX_COLS {
                    return Err(format!("a table has 1 to {MAX_ROWS} rows and 1 to {MAX_COLS} columns"));
                }
                let made = self.edit("insert.table", pick(input, &["rows", "cols", "rect"]))?;
                let id = made.get("id").and_then(Value::as_i64).ok_or("the table was added but no id came back")?;
                if let Some(style) = input.get("style").and_then(Value::as_str) {
                    self.run("table.style", json!({"style": style, "id": id}))?;
                }
                if let Some(grid) = input.get("cells").and_then(Value::as_array) {
                    for (r, row) in grid.iter().take(rows).enumerate() {
                        for (c, cell) in row.as_array().into_iter().flatten().take(cols).enumerate() {
                            let value = match cell {
                                Value::Null => continue,
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            self.run("text.set", json!({"id": id, "text": value, "cell": [r, c]}))?;
                        }
                    }
                }
                Ok(made)
            }
            "add_chart" => {
                let made = self.edit("insert.chart", pick(input, &["type", "categories", "series", "rect"]))?;
                let id = made.get("id").and_then(Value::as_i64).ok_or("the chart was added but no id came back")?;
                let mut options = pick(input, &["title", "legend", "dataLabels"]);
                if options.as_object().is_some_and(|o| !o.is_empty()) {
                    options["id"] = json!(id);
                    self.run("chart.options", options)?;
                }
                Ok(made)
            }
            "add_diagram" => self.edit("insert.smartArt", pick(input, &["kind", "items"])),
            "set_notes" => {
                str_arg(input, "text")?;
                self.edit("slide.notes", pick(input, &["text", "index"]))
            }
            "set_transition" => {
                let set = self.edit("transition.set", pick(input, &["kind", "option", "duration", "index"]))?;
                if input.get("all").and_then(Value::as_bool).unwrap_or(false) {
                    self.run("transition.applyAll", json!({}))?;
                }
                Ok(set)
            }
            "run_command" => {
                let command = str_arg(input, "command")?;
                if forbidden(command) {
                    return Err(format!(
                        "`{command}` is not available to the assistant: files, windows, the slide show and importing files are the person's to handle"
                    ));
                }
                let params = input.get("params").cloned().unwrap_or_else(|| json!({}));
                self.edit(command, params)
            }
            other => Err(format!("there is no tool called `{other}`")),
        }
    }

    fn describe(&self, tool: &str, input: &Value) -> Option<String> {
        let es = self.lang == Lang::Spanish;
        let say = |spanish: String, english: String| if es { spanish } else { english };
        let arg = |k: &str| input.get(k).and_then(Value::as_str).map(|s| clip(s, PREVIEW));
        let count = |k: &str| input.get(k).and_then(Value::as_array).map_or(0, Vec::len);
        let n = |k: &str| input.get(k).and_then(Value::as_u64);
        Some(match tool {
            "add_slide" => {
                let title = arg("title");
                match title {
                    Some(t) => say(format!("Añadir una diapositiva «{t}»"), format!("Add a slide “{t}”")),
                    None => say("Añadir una diapositiva".into(), "Add a slide".into()),
                }
            }
            "set_text" => {
                let t = arg("text")?;
                say(format!("Cambiar el texto a «{t}»"), format!("Change the text to “{t}”"))
            }
            "add_text_box" => {
                let t = arg("text")?;
                say(format!("Añadir un cuadro de texto «{t}»"), format!("Add a text box “{t}”"))
            }
            "add_shape" => {
                let p = arg("preset")?;
                say(format!("Añadir una forma ({p})"), format!("Add a {p} shape"))
            }
            "style_shapes" => {
                let k = count("ids");
                say(format!("Dar estilo a {k} forma(s)"), format!("Style {k} shape(s)"))
            }
            "format_text" => {
                let k = count("ids");
                say(format!("Dar formato al texto de {k} forma(s)"), format!("Format the text of {k} shape(s)"))
            }
            "move_shapes" => {
                let k = count("ids");
                say(format!("Mover o redimensionar {k} forma(s)"), format!("Move or resize {k} shape(s)"))
            }
            "delete_shapes" => {
                let k = count("ids");
                say(format!("Eliminar {k} forma(s)"), format!("Delete {k} shape(s)"))
            }
            "add_table" => {
                let (r, c) = (n("rows")?, n("cols")?);
                say(format!("Añadir una tabla de {r} × {c}"), format!("Add a {r} × {c} table"))
            }
            "add_chart" => {
                let kind = arg("type")?;
                say(format!("Añadir un gráfico ({kind})"), format!("Add a {kind} chart"))
            }
            "add_diagram" => {
                let kind = arg("kind")?;
                say(format!("Añadir un diagrama ({kind})"), format!("Add a {kind} diagram"))
            }
            "set_notes" => say("Escribir las notas del orador".into(), "Write the speaker notes".into()),
            "set_transition" => {
                let kind = arg("kind")?;
                if input.get("all").and_then(Value::as_bool).unwrap_or(false) {
                    say(format!("Poner la transición «{kind}» a todas las diapositivas"), format!("Set the “{kind}” transition on every slide"))
                } else {
                    say(format!("Poner la transición «{kind}»"), format!("Set the “{kind}” transition"))
                }
            }
            "run_command" => {
                let c = arg("command")?;
                let params = input.get("params").filter(|p| p.as_object().is_some_and(|o| !o.is_empty()));
                let params = params.map(|p| clip(&p.to_string(), PREVIEW)).unwrap_or_default();
                let label = self.app.session.commands().into_iter().find(|x| x.id == c).map(|x| x.label.to_string());
                let name = label.map_or_else(|| c.clone(), |l| format!("{l} ({c})"));
                match (es, params.is_empty()) {
                    (true, true) => format!("Ejecutar {name}"),
                    (true, false) => format!("Ejecutar {name} {params}"),
                    (false, true) => format!("Run {name}"),
                    (false, false) => format!("Run {name} {params}"),
                }
            }
            _ => return None,
        })
    }

    fn suggestions(&self, lang: Lang) -> Vec<String> {
        let s: [&str; 4] = if lang == Lang::Spanish {
            [
                "Resume esta presentación en tres frases",
                "Añade una diapositiva de conclusiones",
                "Escribe las notas del orador de cada diapositiva",
                "Mejora el diseño de la diapositiva actual",
            ]
        } else {
            [
                "Summarise this deck in three sentences",
                "Add a closing slide with the key takeaways",
                "Write speaker notes for every slide",
                "Improve the layout of the current slide",
            ]
        };
        s.iter().map(|s| (*s).to_string()).collect()
    }

    fn undo_depth(&mut self) -> usize {
        self.app.session.active().map_or(0, |d| d.history.undo.len())
    }

    fn undo(&mut self, steps: usize) -> Result<(), String> {
        for _ in 0..steps {
            let before = self.undo_depth();
            self.run("edit.undo", json!({}))?;
            if self.undo_depth() >= before {
                return Err("nothing to undo".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
