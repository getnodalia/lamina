# Next-level plan (2026-11-25)

What Lámina already does well, where it still loses to Keynote and PowerPoint, and what the next
release (0.3.0) takes on. The ranked shortfalls are in [gaps.md](gaps.md); this page is the
decision: what is built now, in this order, and what waits.

## Audit

Already in: masters and layouts, Design Ideas (offline, rule-based), 12 original starter decks in
English and Spanish, 48 transitions including Morph, 64 animation effects with triggers,
presenter view with notes and timer, smart guides and grid, comments with replies, tables, charts
with data editing, SmartArt, Insert Icons, spelling, compare, PPTX read and write, PDF and GIF
export, an agent-drivable command registry (CLI, control channel, MCP).

Where someone switching from PowerPoint or Keynote still stops (from `gaps.md`, by how often it
bites):

1. **SVG pictures draw nothing.** Icons and logos are SVG today. Insert accepts the file, then shows an empty frame.
2. **Charts can't take their numbers from a spreadsheet.** Business decks are built from sheets; Celdas saves `.xlsx`. Nothing connects the two.
3. **Printing only through a PDF viewer.** Handouts and speaker notes are still printed in many workplaces; there was no direct way to a printer.
4. **Starter decks are few and plain.** Twelve is a start; the gallery is the first thing a new user sees.
5. PDF artwork is raster; fonts have only Regular/Bold/Italic; equations, video export, a presenter on the projector, password protection, legacy `.ppt`.

## Decision: 0.3.0 takes the first four

Ranked by value for a person who presents, and by what can be proven in tests here.

| # | Work | Why first | Proof |
|---|---|---|---|
| 1 | SVG pictures: draw, size, keep (and PowerPoint's SVG-with-PNG-fallback icons draw from the SVG) | Every icon set and logo is SVG; it is a beta blocker (gap 4) | Render tests against known shapes, size and bounds tests, hostile input |
| 2 | Chart from a spreadsheet range (`.xlsx`, `.csv`, `.tsv`) with Update from source | The Office-suite promise: Celdas to Lámina without retyping | Command tests over generated workbooks; refresh keeps formatting |
| 3 | Print (slides, handouts, notes, a range, copies) straight to the system queue, PDF viewer as fallback | Closes the biggest everyday hole after SVG; reuses the PDF path | Job building, range and layout tests; the queue itself is not run in CI |
| 4 | More starter decks, calm and in Nodalia's look, English and Spanish | First impression; costs no risk | Every deck builds, has text in both languages, fits its layouts |

All of it is a command first (`insert.chart.fromSheet`, `file.print`, …), so the UI, CLI, control
channel and MCP get it together. No gradients or glows; colours come from theme tokens; every
string is translated to Spanish in the same change.

## After 0.3.0

Vector PDF artwork; font weights (#69); equations on `nodalia-math`; presenter view on the second
display; MP4 export; a live link that re-reads a Celdas workbook when it changes; per-object
review of comments.
