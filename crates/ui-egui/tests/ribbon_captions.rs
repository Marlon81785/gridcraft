//! Ribbon captions belong to their buttons, and chrome text is not selectable (#929).

use egui::text_selection::LabelSelectionState;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Shape, vec2};
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness() -> Harness {
    let mut session = Session::new();
    session.new_workbook();
    session.run("range.setValues", json!({"range": "A1", "values": [["title"]]})).unwrap();
    session.run("selection.set", json!({"range": "A1:C1"})).unwrap();
    let mut ready = false;
    let mut h = Harness::builder().with_size(vec2(1600.0, 800.0)).build_ui_state(
        move |ui, app: &mut SheetApp| {
            // The app's own one-time setup, so the test sees the style the window gets.
            if !ready {
                SheetApp::setup_context_for_language(ui.ctx(), false, gridcraft_ui_egui::i18n::Language::En);
                ready = true;
                return;
            }
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        SheetApp::new(session, Default::default()),
    );
    h.run_steps(6);
    h
}

fn text_rect(h: &Harness, text: &str) -> Rect {
    h.output()
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Text(t) if t.galley.job.text == text => Some(t.visual_bounding_rect()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{text:?} was not painted"))
}

fn click(h: &mut Harness, at: Pos2) {
    h.input_mut().events.extend([
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
    ]);
    h.step();
    h.input_mut().events.push(Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(6);
}

fn drag(h: &mut Harness, from: Pos2, to: Pos2) {
    h.input_mut().events.extend([
        Event::PointerMoved(from),
        Event::PointerButton { pos: from, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
    ]);
    h.step();
    for i in 1..=8 {
        h.input_mut().events.push(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    h.input_mut().events.push(Event::PointerButton { pos: to, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(6);
}

fn text_selected(h: &Harness) -> bool {
    h.ctx.with_plugin(|s: &mut LabelSelectionState| s.has_selection()).unwrap_or(false)
}

#[test]
fn clicking_the_merge_and_center_caption_merges() {
    let mut h = harness();
    let caption = text_rect(&h, "Merge & Center");
    click(&mut h, caption.center());
    let doc = h.state().session.active().unwrap();
    let merge = doc.wb.active().unwrap().merge_at(CellRef::parse("A1").unwrap());
    assert_eq!(merge.map(|r| r.to_string()), Some("A1:C1".to_string()), "the caption must run the button's action");
}

#[test]
fn clicking_the_wrap_text_caption_wraps() {
    let mut h = harness();
    let caption = text_rect(&h, "Wrap Text");
    click(&mut h, caption.center());
    let doc = h.state().session.active().unwrap();
    let sheet = doc.wb.active().unwrap();
    assert!(doc.wb.styles.get(sheet.style_id(CellRef::parse("A1").unwrap())).align.wrap, "the caption must run the button's action");
}

#[test]
fn dragging_across_ribbon_captions_selects_no_text() {
    for text in ["Merge & Center", "Wrap Text"] {
        let mut h = harness();
        let caption = text_rect(&h, text);
        drag(&mut h, caption.left_center() + vec2(1.0, 0.0), caption.right_center() - vec2(1.0, 0.0));
        assert!(!text_selected(&h), "dragging across {text:?} selected it as text");
    }
}

#[test]
fn dragging_across_status_bar_text_selects_nothing() {
    let mut h = harness();
    let ready = text_rect(&h, "Ready");
    drag(&mut h, ready.left_center() + vec2(1.0, 0.0), ready.right_center() - vec2(1.0, 0.0));
    assert!(!text_selected(&h), "the status bar's mode label was selected as text");
}
