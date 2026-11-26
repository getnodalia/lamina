#![allow(clippy::unwrap_used, clippy::expect_used)]

use lamina_engine::Session;

use super::*;
use crate::Services;

fn app() -> SlideApp {
    SlideApp::new(Session::with_new(), Services::default())
}

fn host(app: &mut SlideApp) -> LaminaHost<'_> {
    LaminaHost { app, lang: Lang::English }
}

/// The shapes of the current slide, as the model reads them.
fn shapes(h: &mut LaminaHost<'_>) -> Vec<Value> {
    h.call("read_slide", &json!({})).unwrap()["shapes"].as_array().cloned().unwrap_or_default()
}

fn find_text(shapes: &[Value], text: &str) -> Option<Value> {
    shapes.iter().find(|s| s["text"].as_str().is_some_and(|t| t.contains(text))).cloned()
}

/// The tools the model is offered: every one named once, each schema an object, and only the
/// ones that leave the deck alone run without being asked about.
#[test]
fn the_tools_are_named_once_and_only_readers_run_unasked() {
    let mut a = app();
    let tools = host(&mut a).tools();
    let mut names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "a tool is listed twice");
    for t in &tools {
        assert_eq!(t.schema["type"], "object", "{}", t.name);
        assert!(!t.description.is_empty(), "{}", t.name);
    }
    for edit in [
        "add_slide",
        "set_text",
        "add_text_box",
        "add_shape",
        "style_shapes",
        "format_text",
        "move_shapes",
        "delete_shapes",
        "add_table",
        "add_chart",
        "add_diagram",
        "set_notes",
        "set_transition",
        "run_command",
    ] {
        assert!(tools.iter().any(|t| t.name == edit && !t.read_only), "{edit} must wait for the person");
    }
    for read in ["read_slide", "read_shape", "go_to_slide", "check_accessibility", "list_commands"] {
        assert!(tools.iter().any(|t| t.name == read && t.read_only), "{read} is a reader");
    }
}

/// A slide built from a layout shows up in the snapshot, and a whole turn undoes by the number
/// of steps it made.
#[test]
fn building_a_slide_changes_the_deck_and_undo_takes_it_back() {
    let mut a = app();
    let mut h = host(&mut a);
    let start = h.undo_depth();
    let slides_before = h.context().unwrap()["slides"].as_array().unwrap().len();
    h.call("add_slide", &json!({"layout": "titleAndContent", "title": "Quarterly results", "body": "Revenue up\nCosts down"})).unwrap();
    h.call("set_notes", &json!({"text": "Open with the revenue figure."})).unwrap();
    let depth = h.undo_depth() - start;
    assert!(depth >= 2, "each change is a step, got {depth}");
    let snapshot = h.context().unwrap();
    assert_eq!(snapshot["slides"].as_array().unwrap().len(), slides_before + 1);
    assert!(snapshot.get("undo").is_none(), "undo labels are not part of the snapshot");
    let current = &snapshot["currentSlide"];
    assert_eq!(current["title"], "Quarterly results");
    assert!(current.get("notesBody").is_none(), "the notes' formatting stays out of the snapshot");
    let found = shapes(&mut h);
    assert!(find_text(&found, "Revenue up").is_some());
    h.undo(depth).unwrap();
    assert_eq!(h.context().unwrap()["slides"].as_array().unwrap().len(), slides_before);
}

/// Shapes, text boxes, styling, formatting, moving and deleting go through the commands.
#[test]
fn shapes_can_be_added_styled_moved_and_deleted() {
    let mut a = app();
    let mut h = host(&mut a);
    let made = h
        .call("add_shape", &json!({"preset": "roundRect", "rect": [100, 100, 200, 80], "text": "Launch", "fill": "#336699", "outline": "none"}))
        .unwrap();
    let id = made["id"].as_i64().unwrap();
    let shape = h.call("read_shape", &json!({"id": id})).unwrap();
    assert_eq!(shape["shape"]["fill"]["color"]["base"]["rgb"]["r"], 0x33, "the fill was applied: {shape}");
    assert_eq!(shape["shape"]["line"]["fill"]["kind"], "none", "the outline was removed");
    let applied = h.call("format_text", &json!({"ids": [id], "bold": true, "size": 28, "align": "center"})).unwrap();
    assert_eq!(applied["applied"].as_array().unwrap().len(), 3);
    h.call("move_shapes", &json!({"ids": [id], "x": 300, "y": 200, "w": 240})).unwrap();
    let moved = h.call("read_shape", &json!({"id": id})).unwrap();
    assert!(moved.to_string().contains("300"), "{moved}");
    let text_box = h.call("add_text_box", &json!({"text": "Footnote", "rect": [40, 480, 300, 30]})).unwrap();
    assert!(find_text(&shapes(&mut h), "Footnote").is_some());
    h.call("set_text", &json!({"id": text_box["id"], "text": "Source: finance"})).unwrap();
    assert!(find_text(&shapes(&mut h), "Source: finance").is_some());
    h.call("delete_shapes", &json!({"ids": [id, text_box["id"]]})).unwrap();
    let left = shapes(&mut h);
    assert!(find_text(&left, "Launch").is_none() && find_text(&left, "Source").is_none());
}

/// Tables, charts and diagrams come with their content.
#[test]
fn a_table_a_chart_and_a_diagram_are_built_with_their_content() {
    let mut a = app();
    let mut h = host(&mut a);
    let table =
        h.call("add_table", &json!({"rows": 2, "cols": 2, "cells": [["Region", "Sales"], ["North", 120]], "style": "medium2-accent1"})).unwrap();
    let id = table["id"].as_i64().unwrap();
    let text = h.call("read_shape", &json!({"id": id})).unwrap().to_string();
    assert!(text.contains("North") && text.contains("120"), "{text}");
    let chart = h
        .call(
            "add_chart",
            &json!({"type": "column", "categories": ["Q1", "Q2"], "series": [{"name": "Sales", "values": [3, 5]}], "title": "Sales", "legend": "b"}),
        )
        .unwrap();
    assert!(chart["id"].as_i64().is_some());
    let diagram = h.call("add_diagram", &json!({"kind": "process", "items": ["Plan", "Build", "Ship"]})).unwrap();
    assert!(diagram["id"].as_i64().is_some());
    assert!(h.call("add_table", &json!({"rows": 0, "cols": 3})).is_err());
    assert!(h.call("add_table", &json!({"rows": 500, "cols": 3})).is_err(), "a table has a size limit");
}

/// Files, windows, the show and importing from the disk are not the assistant's to touch.
#[test]
fn forbidden_commands_are_refused_and_hidden() {
    let mut a = app();
    let mut h = host(&mut a);
    for command in [
        "file.save",
        "file.open",
        "ui.dialog",
        "window.next",
        "app.quit",
        "show.fromStart",
        "insert.picture",
        "insert.video",
        "view.slideMaster",
        "edit.copy",
    ] {
        let err = h.call("run_command", &json!({"command": command})).unwrap_err();
        assert!(err.contains("not available"), "{command}: {err}");
    }
    let listed = h.call("list_commands", &json!({})).unwrap();
    let commands = listed["commands"].as_array().unwrap();
    assert!(!commands.is_empty());
    assert!(commands.iter().all(|c| !forbidden(c["id"].as_str().unwrap_or_default())));
    let found = h.call("list_commands", &json!({"query": "transition"})).unwrap();
    assert!(found["commands"].as_array().unwrap().iter().any(|c| c["id"] == "transition.set"));
}

/// Commands the assistant may run still work, and the transition reaches every slide on request.
#[test]
fn run_command_and_the_transition_work() {
    let mut a = app();
    let mut h = host(&mut a);
    h.call("add_slide", &json!({"layout": "titleOnly", "title": "Two"})).unwrap();
    h.call("set_transition", &json!({"kind": "fade", "all": true})).unwrap();
    for index in 0..2 {
        let slide = h.call("read_slide", &json!({"index": index})).unwrap();
        assert_eq!(slide["transition"]["kind"], "fade", "slide {index}");
    }
    h.call("go_to_slide", &json!({"index": 0})).unwrap();
    h.call("run_command", &json!({"command": "slide.duplicate"})).unwrap();
    assert_eq!(h.context().unwrap()["slides"].as_array().unwrap().len(), 3);
}

/// Bad input comes back as a message the model can act on, never a panic.
#[test]
fn bad_input_is_an_error_the_model_can_read() {
    let mut a = app();
    let mut h = host(&mut a);
    assert!(h.call("set_text", &json!({"id": 1})).is_err());
    assert!(h.call("set_text", &json!({"text": "x"})).is_err());
    assert!(h.call("add_text_box", &json!({})).is_err());
    assert!(h.call("format_text", &json!({"ids": [1]})).unwrap_err().contains("nothing to apply"));
    assert!(h.call("format_text", &json!({"ids": [1], "align": "diagonal"})).is_err());
    assert!(h.call("style_shapes", &json!({"ids": [1]})).unwrap_err().contains("nothing to apply"));
    assert!(h.call("move_shapes", &json!({"ids": []})).is_err());
    assert!(h.call("move_shapes", &json!({"ids": [1]})).unwrap_err().contains("nothing to change"));
    assert!(h.call("style_shapes", &json!({"ids": [999_999], "fill": "#FF0000"})).is_err(), "a shape that is not there");
    assert!(h.call("run_command", &json!({"command": "no.such"})).is_err());
    assert!(h.call("nothing", &json!({})).unwrap_err().contains("no tool"));
    assert!(h.undo(1).is_err(), "nothing to undo in a new deck");
}

/// The approval card and the suggestions speak both languages.
#[test]
fn descriptions_and_suggestions_are_in_both_languages() {
    let mut a = app();
    let mut h = host(&mut a);
    let slide = json!({"title": "Roadmap"});
    assert_eq!(h.describe("add_slide", &slide).unwrap(), "Add a slide “Roadmap”");
    assert_eq!(h.describe("add_table", &json!({"rows": 3, "cols": 2})).unwrap(), "Add a 3 × 2 table");
    h.lang = Lang::Spanish;
    assert_eq!(h.describe("add_slide", &slide).unwrap(), "Añadir una diapositiva «Roadmap»");
    assert_eq!(h.describe("add_table", &json!({"rows": 3, "cols": 2})).unwrap(), "Añadir una tabla de 3 × 2");
    assert!(h.describe("run_command", &json!({"command": "slide.duplicate"})).unwrap().starts_with("Ejecutar "));
    assert!(h.describe("read_slide", &json!({})).is_none());
    assert_eq!(h.suggestions(Lang::Spanish).len(), 4);
    assert_eq!(h.suggestions(Lang::English).len(), 4);
}

/// The panel and the button draw inside the real window, open and closed.
#[test]
fn the_button_and_the_panel_draw_in_a_frame() {
    let ctx = egui::Context::default();
    let mut a = app();
    let frame = |a: &mut SlideApp| {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            a.logic(ui.ctx());
            a.ui(ui);
        })
        .drop_without_applying_deltas();
    };
    for _ in 0..4 {
        frame(&mut a);
    }
    assert!(a.assistant.is_some(), "the title bar made the assistant");
    for open in [true, false] {
        if let Some(x) = a.assistant.as_mut() {
            x.open = open;
        }
        frame(&mut a);
        frame(&mut a);
    }
}
