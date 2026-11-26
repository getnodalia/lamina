//! File › Print to a printer: the job is checked and described without a printer being there
//! (`dryRun`), since the spooler itself is not something a test can drive.

use lamina_engine::Session;
use serde_json::json;

fn deck(slides: usize) -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({})).expect("new deck");
    for _ in 1..slides {
        s.execute("slide.new", &json!({})).expect("slide");
    }
    s
}

#[test]
fn a_dry_run_counts_the_slides_and_names_the_job() {
    let mut s = deck(5);
    let r = s.execute("file.print", &json!({"dryRun": true, "copies": 3, "slides": "2-4", "printer": "Office"})).expect("dry run");
    assert_eq!(r["dryRun"], true);
    assert_eq!(r["slides"], 3);
    assert_eq!(r["copies"], 3);
    if cfg!(unix) {
        let cmd: Vec<String> = serde_json::from_value(r["command"].clone()).expect("command");
        let line = cmd.join(" ");
        assert!(line.starts_with("lp -d Office -n 3 "), "{line}");
        assert!(line.contains("collate=true"), "{line}");
    }
}

#[test]
fn every_slide_prints_by_default_and_hidden_ones_stay_out() {
    let mut s = deck(4);
    assert_eq!(s.execute("file.print", &json!({"dryRun": true})).expect("all")["slides"], 4);
    s.execute("slide.hide", &json!({"slide": 1})).expect("hide");
    assert_eq!(s.execute("file.print", &json!({"dryRun": true})).expect("without hidden")["slides"], 3);
    assert_eq!(s.execute("file.print", &json!({"dryRun": true, "includeHidden": true})).expect("with hidden")["slides"], 4);
}

#[test]
fn two_sided_greyscale_and_uncollated_reach_the_spooler_options() {
    if !cfg!(unix) {
        return;
    }
    let mut s = deck(2);
    let r = s
        .execute("file.print", &json!({"dryRun": true, "duplex": "shortEdge", "grayscale": true, "collate": false, "scaling": "shrink"}))
        .expect("dry run");
    let line = serde_json::from_value::<Vec<String>>(r["command"].clone()).expect("command").join(" ");
    for want in ["sides=two-sided-short-edge", "print-color-mode=monochrome", "collate=false", "print-scaling=auto-fit"] {
        assert!(line.contains(want), "{want} in {line}");
    }
}

#[test]
fn handouts_and_notes_print_too() {
    let mut s = deck(7);
    for p in [json!({"layout": "notes"}), json!({"layout": "handouts", "perPage": 3, "paper": "a4", "orientation": "landscape"})] {
        let mut p = p;
        p["dryRun"] = json!(true);
        assert_eq!(s.execute("file.print", &p).expect("layout")["slides"], 7);
    }
}

#[test]
fn bad_jobs_are_refused_with_the_reason() {
    let mut s = deck(3);
    for (p, why) in [
        (json!({"dryRun": true, "copies": 0}), "copies"),
        (json!({"dryRun": true, "copies": 5000}), "copies"),
        (json!({"dryRun": true, "duplex": "diagonal"}), "duplex"),
        (json!({"dryRun": true, "slides": "9"}), "slide"),
        (json!({"dryRun": true, "slides": []}), "no slides"),
        (json!({"dryRun": true, "layout": "handouts", "perPage": 5}), "perPage"),
        (json!({"dryRun": true, "paper": "tabloid-xl"}), "paper"),
        (json!({"dryRun": true, "printer": ["a"]}), "printer"),
    ] {
        let e = s.execute("file.print", &p).expect_err(&p.to_string()).to_string();
        assert!(e.contains(why), "{p}: {e}");
    }
}

#[test]
fn the_printers_command_answers_without_a_spooler() {
    // A machine with no spooler or no printers lists none; it never fails.
    let r = Session::new().execute("file.printers", &json!({})).expect("printers");
    assert!(r["printers"].is_array());
    assert_eq!(r["available"], cfg!(not(target_arch = "wasm32")));
}
