//! ParaKeys GUI design system.
//!
//! Tokens, motion, surfaces, and interactive widgets in one place so screens
//! stay fluid and consistent. Call sites should prefer these helpers over raw
//! colors, padding, or one-off paint code.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Id, Margin, RichText, Sense, Shadow, Stroke, Ui,
    Vec2,
};

// ─── Scale ───────────────────────────────────────────────────────────────────

/// 4pt base spacing scale.
#[derive(Clone, Copy)]
pub struct Space;
impl Space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 24.0;
    pub const XXXL: f32 = 32.0;
}

#[derive(Clone, Copy)]
pub struct Radius;
impl Radius {
    pub const SM: u8 = 6;
    pub const MD: u8 = 10;
    pub const LG: u8 = 14;
    pub const XL: u8 = 18;
    pub const XXL: u8 = 22;
    pub const PILL: u8 = 100;
}

#[derive(Clone, Copy)]
pub struct Type;
impl Type {
    pub const MICRO: f32 = 10.0;
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 13.0;
    pub const CALL_OUT: f32 = 14.0;
    pub const TITLE: f32 = 15.0;
    pub const HEADLINE: f32 = 20.0;
    pub const HERO: f32 = 26.0;
}

/// Motion durations (seconds). Pair with [`anim`] helpers for soft transitions.
#[derive(Clone, Copy)]
pub struct Motion;
impl Motion {
    pub const INSTANT: f32 = 0.0;
    pub const FAST: f32 = 0.10;
    pub const NORMAL: f32 = 0.16;
    pub const SLOW: f32 = 0.28;
}

// ─── Semantic color ──────────────────────────────────────────────────────────

/// Light product palette. Prefer semantic names over raw hex at call sites.
#[derive(Clone, Copy)]
pub struct Color;
impl Color {
    pub const BG: Color32 = Color32::from_rgb(240, 240, 245);
    pub const BG_SIDE: Color32 = Color32::from_rgb(246, 246, 249);
    pub const SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const SURFACE_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);
    pub const FILL: Color32 = Color32::from_rgb(232, 232, 237);
    pub const FILL_HOVER: Color32 = Color32::from_rgb(222, 222, 228);
    pub const FILL_ACTIVE: Color32 = Color32::from_rgb(210, 210, 218);
    pub const BORDER: Color32 = Color32::from_rgb(218, 218, 224);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(232, 232, 236);
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(190, 190, 198);
    pub const TEXT: Color32 = Color32::from_rgb(28, 28, 30);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(110, 110, 115);
    pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(142, 142, 147);
    pub const TEXT_ON_ACCENT: Color32 = Color32::WHITE;
    pub const ACCENT: Color32 = Color32::from_rgb(0, 122, 255);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(10, 132, 255);
    pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(0, 100, 220);
    pub const ACCENT_SOFT: Color32 = Color32::from_rgb(232, 242, 255);
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 199, 89);
    pub const SUCCESS_SOFT: Color32 = Color32::from_rgb(232, 248, 237);
    pub const WARNING: Color32 = Color32::from_rgb(255, 149, 0);
    pub const WARNING_SOFT: Color32 = Color32::from_rgb(255, 246, 230);
    pub const DANGER: Color32 = Color32::from_rgb(255, 59, 48);
    pub const DANGER_SOFT: Color32 = Color32::from_rgb(255, 236, 234);
    pub const SELECTION: Color32 = Color32::from_rgb(0, 122, 255);
    pub const WARNING_BG: Color32 = Color32::from_rgb(255, 250, 240);
    pub const WARNING_BORDER: Color32 = Color32::from_rgb(230, 200, 160);
}

// ─── Layout constants ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Layout;
impl Layout {
    pub const SIDEBAR_W: f32 = 236.0;
    pub const LIST_W: f32 = 292.0;
    pub const TOOLBAR_H: f32 = 54.0;
    pub const TILE: Vec2 = Vec2::new(102.0, 78.0);
    pub const ROW_MIN_H: f32 = 54.0;
    pub const CONTROL_H: f32 = 30.0;
    pub const CONTROL_H_SM: f32 = 26.0;
    pub const ICON_SM: f32 = 22.0;
    pub const ICON_MD: f32 = 34.0;
    pub const ICON_LG: f32 = 76.0;
    pub const FOCUS_RING: f32 = 2.0;
}

// ─── Color utilities ─────────────────────────────────────────────────────────

pub fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

pub fn soft_fill(accent: Color32, strength: f32) -> Color32 {
    mix(Color::SURFACE, accent, strength.clamp(0.0, 1.0))
}

// ─── Elevation ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum Elevation {
    Flat,
    Raised,
    Floating,
}

impl Elevation {
    pub fn shadow(self) -> Shadow {
        match self {
            Self::Flat => Shadow::NONE,
            Self::Raised => Shadow {
                offset: [0, 1],
                blur: 6,
                spread: 0,
                color: with_alpha(Color32::BLACK, 18),
            },
            Self::Floating => Shadow {
                offset: [0, 4],
                blur: 18,
                spread: 0,
                color: with_alpha(Color32::BLACK, 28),
            },
        }
    }
}

// ─── Interactive state ───────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InteractState {
    Idle,
    Hovered,
    Pressed,
    Selected,
    Disabled,
}

impl InteractState {
    pub fn from_response(resp: &egui::Response, selected: bool, enabled: bool) -> Self {
        if !enabled {
            Self::Disabled
        } else if selected {
            Self::Selected
        } else if resp.is_pointer_button_down_on() {
            Self::Pressed
        } else if resp.hovered() {
            Self::Hovered
        } else {
            Self::Idle
        }
    }
}

/// Smooth 0..1 transition for a boolean interaction (hover, select, open).
pub fn anim(ui: &Ui, id: impl Into<Id>, on: bool) -> f32 {
    ui.ctx()
        .animate_bool_with_time_and_easing(id.into(), on, Motion::NORMAL, emath_cubic_out)
}

pub fn anim_fast(ui: &Ui, id: impl Into<Id>, on: bool) -> f32 {
    ui.ctx()
        .animate_bool_with_time_and_easing(id.into(), on, Motion::FAST, emath_cubic_out)
}

fn emath_cubic_out(t: f32) -> f32 {
    // cubic_out: 1 - (1-t)^3
    let u = 1.0 - t;
    1.0 - u * u * u
}

// ─── Theme application ───────────────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill = Color::BG;
    v.window_fill = Color::BG;
    v.override_text_color = Some(Color::TEXT);
    v.widgets.noninteractive.bg_fill = Color::SURFACE;
    v.widgets.inactive.bg_fill = Color::FILL;
    v.widgets.hovered.bg_fill = Color::FILL_HOVER;
    v.widgets.active.bg_fill = Color::ACCENT_PRESSED;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color::BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color::TEXT);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color::TEXT);
    v.selection.bg_fill = Color::SELECTION;
    v.selection.stroke = Stroke::new(1.0, Color::ACCENT);
    v.extreme_bg_color = Color::SURFACE;
    v.faint_bg_color = Color::FILL;
    v.window_corner_radius = CornerRadius::same(0);
    v.menu_corner_radius = CornerRadius::same(Radius::MD);
    v.window_shadow = Elevation::Floating.shadow();
    v.popup_shadow = Elevation::Raised.shadow();
    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(Space::SM, Space::SM);
    style.spacing.button_padding = Vec2::new(Space::MD, 7.0);
    style.spacing.indent = Space::LG;
    style.interaction.selectable_labels = false;
    style.interaction.tooltip_delay = 0.35;
    style.animation_time = Motion::NORMAL;
    ctx.set_style(style);
}

// ─── Typography helpers ──────────────────────────────────────────────────────

pub fn text_hero(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::HERO).strong().color(Color::TEXT)
}
pub fn text_headline(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .size(Type::HEADLINE)
        .strong()
        .color(Color::TEXT)
}
pub fn text_title(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::TITLE).strong().color(Color::TEXT)
}
pub fn text_body(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::BODY).color(Color::TEXT)
}
pub fn text_body_secondary(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::BODY).color(Color::TEXT_SECONDARY)
}
pub fn text_caption(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::CAPTION).color(Color::TEXT_TERTIARY)
}
pub fn text_section(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .size(Type::CAPTION)
        .strong()
        .color(Color::TEXT_TERTIARY)
}
pub fn text_mono(s: impl Into<String>, size: f32) -> RichText {
    RichText::new(s)
        .size(size)
        .monospace()
        .color(Color::TEXT_SECONDARY)
}

// ─── Surfaces ────────────────────────────────────────────────────────────────

pub fn panel_side() -> Frame {
    Frame::new()
        .fill(Color::BG_SIDE)
        .inner_margin(Margin::symmetric(Space::MD as i8, Space::MD as i8))
}

pub fn panel_list() -> Frame {
    Frame::new()
        .fill(Color::SURFACE)
        .stroke(Stroke::new(1.0, Color::BORDER))
        .inner_margin(Margin::ZERO)
}

pub fn panel_main() -> Frame {
    Frame::new()
        .fill(Color::BG)
        .inner_margin(Margin::symmetric(Space::XXL as i8, Space::XL as i8))
}

pub fn panel_toolbar() -> Frame {
    Frame::new()
        .fill(mix(Color::BG, Color::SURFACE, 0.55))
        .stroke(Stroke::new(1.0, Color::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(Space::LG as i8, Space::SM as i8 + 2))
}

pub fn card() -> Frame {
    Frame::new()
        .fill(Color::SURFACE)
        .stroke(Stroke::new(1.0, Color::BORDER))
        .corner_radius(CornerRadius::same(Radius::MD))
        .shadow(Elevation::Raised.shadow())
        .inner_margin(Margin::symmetric(Space::LG as i8, Space::SM as i8))
}

pub fn card_flat() -> Frame {
    Frame::new()
        .fill(Color::SURFACE)
        .stroke(Stroke::new(1.0, Color::BORDER))
        .corner_radius(CornerRadius::same(Radius::MD))
        .inner_margin(Margin::symmetric(Space::LG as i8, Space::SM as i8))
}

pub fn card_warning() -> Frame {
    Frame::new()
        .fill(Color::WARNING_BG)
        .stroke(Stroke::new(1.0, Color::WARNING_BORDER))
        .corner_radius(CornerRadius::same(Radius::MD))
        .shadow(Elevation::Raised.shadow())
        .inner_margin(Margin::same(Space::MD as i8))
}

// ─── Controls ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
    Pill,
}

pub fn button(ui: &mut Ui, label: &str, kind: ButtonKind) -> egui::Response {
    let id = ui.next_auto_id();
    let desired = Vec2::new(0.0, Layout::CONTROL_H);

    // Pre-layout text for sizing, then paint fluid fill.
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(Type::BODY),
        Color::TEXT,
    );
    let pad_x = match kind {
        ButtonKind::Pill => Space::MD,
        _ => Space::LG,
    };
    let size = Vec2::new(
        (galley.size().x + pad_x * 2.0).max(Layout::CONTROL_H),
        Layout::CONTROL_H,
    );
    let (rect, resp) = ui.allocate_exact_size(size.max(desired), Sense::click());
    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();
    let t = anim_fast(ui, id.with("btn"), hovered || pressed);

    let (bg, fg, stroke, radius) = match kind {
        ButtonKind::Primary => {
            let base = if pressed {
                Color::ACCENT_PRESSED
            } else {
                mix(Color::ACCENT, Color::ACCENT_HOVER, t)
            };
            (
                base,
                Color::TEXT_ON_ACCENT,
                Stroke::NONE,
                CornerRadius::same(Radius::SM),
            )
        }
        ButtonKind::Secondary => {
            let base = mix(Color::FILL, Color::FILL_HOVER, t);
            let base = if pressed {
                Color::FILL_ACTIVE
            } else {
                base
            };
            (
                base,
                Color::TEXT,
                Stroke::NONE,
                CornerRadius::same(Radius::SM),
            )
        }
        ButtonKind::Ghost => {
            let base = mix(Color32::TRANSPARENT, Color::FILL, t * 0.9);
            (
                base,
                Color::TEXT,
                Stroke::NONE,
                CornerRadius::same(Radius::SM),
            )
        }
        ButtonKind::Danger => {
            let base = if pressed {
                mix(Color::DANGER, Color32::BLACK, 0.12)
            } else {
                mix(Color::DANGER, Color::DANGER, t)
            };
            (
                base,
                Color::TEXT_ON_ACCENT,
                Stroke::NONE,
                CornerRadius::same(Radius::SM),
            )
        }
        ButtonKind::Pill => {
            let base = mix(Color::SURFACE, Color::FILL, t);
            (
                base,
                Color::TEXT,
                Stroke::new(1.0, mix(Color::BORDER, Color::BORDER_STRONG, t)),
                CornerRadius::same(Radius::PILL),
            )
        }
    };

    ui.painter()
        .rect(rect, radius, bg, stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(Type::BODY),
        fg,
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn primary_button(ui: &mut Ui, label: &str) -> egui::Response {
    button(ui, label, ButtonKind::Primary)
}

pub fn secondary_button(ui: &mut Ui, label: &str) -> egui::Response {
    button(ui, label, ButtonKind::Secondary)
}

pub fn toolbar_pill(ui: &mut Ui, label: &str) -> egui::Response {
    button(ui, label, ButtonKind::Pill)
}

pub fn search_field(ui: &mut Ui, text: &mut String, width: f32) -> egui::Response {
    let id = ui.next_auto_id();
    let mut resp = None;
    let focused_hint = ui.memory(|m| m.has_focus(id));

    let t = anim(ui, id.with("search_focus"), focused_hint);
    let border = mix(Color::BORDER, Color::ACCENT, t * 0.85);
    let fill = mix(Color::SURFACE, Color::ACCENT_SOFT, t * 0.35);

    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(Radius::PILL))
        .shadow(if t > 0.05 {
            Elevation::Raised.shadow()
        } else {
            Shadow::NONE
        })
        .inner_margin(Margin::symmetric(Space::MD as i8, 6))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⌕")
                        .size(Type::BODY)
                        .color(mix(Color::TEXT_TERTIARY, Color::ACCENT, t)),
                );
                let te = ui.add(
                    egui::TextEdit::singleline(text)
                        .id(id)
                        .hint_text("Search")
                        .frame(false)
                        .desired_width(width - 36.0)
                        .font(FontId::proportional(Type::BODY)),
                );
                resp = Some(te);
            });
        });
    resp.unwrap()
}

pub fn section_label(ui: &mut Ui, label: &str) {
    ui.add_space(Space::MD);
    ui.label(text_section(label.to_ascii_uppercase()));
    ui.add_space(Space::XS);
}

/// Soft glyph badge used in rows, tiles, and detail heroes.
pub fn icon_badge(
    ui: &mut Ui,
    size: f32,
    glyph: &str,
    accent: Color32,
    selected: bool,
    radius: u8,
) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let bg = if selected {
        with_alpha(Color32::WHITE, 40)
    } else {
        soft_fill(accent, 0.18)
    };
    let fg = if selected {
        Color::TEXT_ON_ACCENT
    } else {
        accent
    };
    let corner = if radius >= Radius::PILL / 2 {
        CornerRadius::same(Radius::PILL)
    } else {
        CornerRadius::same(radius)
    };
    ui.painter()
        .rect(r, corner, bg, Stroke::NONE, egui::StrokeKind::Inside);
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::proportional((size * 0.36).clamp(10.0, 16.0)),
        fg,
    );
}

/// Soft category tile with hover lift and selection animation.
pub fn category_tile(
    ui: &mut Ui,
    title: &str,
    count: usize,
    glyph: &str,
    accent: Color32,
    selected: bool,
) -> egui::Response {
    let id = ui.next_auto_id();
    let (rect, resp) = ui.allocate_exact_size(Layout::TILE, Sense::click());
    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();
    let select_t = anim(ui, id.with("sel"), selected);
    let hover_t = anim_fast(ui, id.with("hov"), hovered && !selected);

    let idle_bg = Color::SURFACE;
    let hover_bg = Color::FILL_HOVER;
    let select_bg = accent;
    let bg = mix(mix(idle_bg, hover_bg, hover_t), select_bg, select_t);
    let stroke_c = mix(
        mix(Color::BORDER, Color::BORDER_STRONG, hover_t),
        with_alpha(accent, 0),
        select_t,
    );
    let stroke = Stroke::new(1.0 - select_t, stroke_c);

    // Slight lift on hover (shadow strength).
    if hover_t > 0.02 || select_t > 0.5 {
        let shadow = Elevation::Raised.shadow();
        let srect = rect.translate(Vec2::new(0.0, hover_t * 1.0));
        ui.painter().add(shadow.as_shape(srect, CornerRadius::same(Radius::LG)));
    }

    let draw_rect = if pressed {
        rect.shrink(0.5)
    } else {
        rect
    };
    ui.painter().rect(
        draw_rect,
        CornerRadius::same(Radius::LG),
        bg,
        stroke,
        egui::StrokeKind::Inside,
    );

    let fg = mix(Color::TEXT, Color::TEXT_ON_ACCENT, select_t);
    let muted = mix(
        Color::TEXT_SECONDARY,
        with_alpha(Color32::WHITE, 200),
        select_t,
    );

    let icon_c = draw_rect.left_top() + Vec2::new(20.0, 24.0);
    let icon_bg = mix(
        accent.gamma_multiply(0.16),
        with_alpha(Color32::WHITE, 40),
        select_t,
    );
    ui.painter().circle_filled(icon_c, 12.0, icon_bg);
    ui.painter().text(
        icon_c,
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(11.0),
        mix(accent, Color::TEXT_ON_ACCENT, select_t),
    );

    ui.painter().text(
        draw_rect.right_top() + Vec2::new(-12.0, 12.0),
        egui::Align2::RIGHT_TOP,
        format!("{count}"),
        FontId::proportional(Type::BODY),
        muted,
    );
    ui.painter().text(
        draw_rect.left_bottom() + Vec2::new(14.0, -12.0),
        egui::Align2::LEFT_BOTTOM,
        title,
        FontId::proportional(Type::BODY),
        fg,
    );

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn list_row(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    glyph: &str,
    accent: Color32,
    selected: bool,
) -> egui::Response {
    let id = ui.next_auto_id();
    let full_w = ui.available_width();
    let height = Layout::ROW_MIN_H;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(full_w, height), Sense::click());
    let hovered = resp.hovered();
    let select_t = anim(ui, id.with("sel"), selected);
    let hover_t = anim_fast(ui, id.with("hov"), hovered && !selected);

    let bg = mix(
        mix(Color32::TRANSPARENT, Color::FILL, hover_t * 0.85),
        Color::SELECTION,
        select_t,
    );
    let inset = rect.shrink2(Vec2::new(Space::SM, 2.0));
    ui.painter().rect(
        inset,
        CornerRadius::same(Radius::MD),
        bg,
        Stroke::NONE,
        egui::StrokeKind::Inside,
    );

    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inset.shrink2(Vec2::new(Space::MD, Space::SM)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    {
        let icon_bg = mix(
            accent.gamma_multiply(0.18),
            with_alpha(Color32::WHITE, 40),
            select_t,
        );
        let icon_fg = mix(accent, Color::TEXT_ON_ACCENT, select_t);
        let (ir, _) = content.allocate_exact_size(Vec2::splat(Layout::ICON_MD), Sense::hover());
        content.painter().circle_filled(ir.center(), 16.0, icon_bg);
        content.painter().text(
            ir.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(12.0),
            icon_fg,
        );
        content.add_space(Space::SM);
        content.vertical(|ui| {
            ui.label(
                RichText::new(title)
                    .size(Type::CALL_OUT)
                    .strong()
                    .color(mix(Color::TEXT, Color::TEXT_ON_ACCENT, select_t)),
            );
            ui.label(
                RichText::new(subtitle).size(Type::CAPTION).color(mix(
                    Color::TEXT_SECONDARY,
                    with_alpha(Color32::WHITE, 200),
                    select_t,
                )),
            );
        });
    }

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn project_card(ui: &mut Ui, name: &str, subtitle: &str, accent: Color32) -> egui::Response {
    let id = ui.next_auto_id();
    let out = card_flat()
        .inner_margin(Margin::symmetric(Space::MD as i8, Space::SM as i8 + 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icon_badge(ui, Layout::ICON_SM + 6.0, "◆", accent, false, Radius::SM);
                ui.add_space(Space::SM);
                ui.vertical(|ui| {
                    ui.label(text_body(name).strong());
                    ui.label(text_caption(subtitle));
                });
            });
        });
    let resp = ui.interact(out.response.rect, id, Sense::click());
    let t = anim_fast(ui, id.with("hov"), resp.hovered());
    if t > 0.01 {
        // Soft border emphasis on hover without full repaint of card fill.
        ui.painter().rect_stroke(
            out.response.rect,
            CornerRadius::same(Radius::MD),
            Stroke::new(1.0, mix(Color::BORDER, Color::ACCENT, t * 0.6)),
            egui::StrokeKind::Inside,
        );
    }
    resp
}

pub fn status_chip(ui: &mut Ui, label: &str, accent: Color32) {
    Frame::new()
        .fill(soft_fill(accent, 0.14))
        .corner_radius(CornerRadius::same(Radius::PILL))
        .inner_margin(Margin::symmetric(Space::SM as i8 + 2, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .size(Type::CAPTION)
                    .strong()
                    .color(accent),
            );
        });
}

pub fn detail_field(ui: &mut Ui, label: &str, value: &str, mono: bool) {
    ui.horizontal(|ui| {
        ui.set_min_height(42.0);
        ui.label(RichText::new(label).size(Type::BODY).color(Color::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut t = RichText::new(value)
                .size(Type::BODY)
                .color(Color::TEXT_SECONDARY);
            if mono {
                t = t.monospace();
            }
            ui.label(t);
        });
    });
}

pub fn hairline(ui: &mut Ui) {
    let y = ui.cursor().top();
    let r = ui.max_rect();
    ui.painter()
        .hline(r.x_range(), y, Stroke::new(1.0, Color::BORDER_SUBTLE));
    ui.add_space(1.0);
}

pub fn empty_state(ui: &mut Ui, title: &str, body: &str, cta: Option<&str>) -> bool {
    let mut clicked = false;
    let h = ui.available_height();
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), h),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space((h * 0.16).clamp(Space::XXL, 96.0));
            let (r, _) = ui.allocate_exact_size(Vec2::splat(Layout::ICON_LG), Sense::hover());
            ui.painter().add(
                Elevation::Raised
                    .shadow()
                    .as_shape(r, CornerRadius::same(Radius::XXL)),
            );
            ui.painter().rect(
                r,
                CornerRadius::same(Radius::XXL),
                Color::ACCENT,
                Stroke::NONE,
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                "key",
                FontId::proportional(15.0),
                Color::TEXT_ON_ACCENT,
            );
            ui.add_space(Space::LG);
            ui.label(text_hero(title));
            ui.add_space(Space::SM);
            ui.label(text_body_secondary(body));
            if let Some(label) = cta {
                ui.add_space(Space::XL);
                if primary_button(ui, &format!("  {label}  ")).clicked() {
                    clicked = true;
                }
            }
        },
    );
    clicked
}

pub fn status_accent(status_kind: u8) -> (Color32, &'static str) {
    // 0 set, 1 missing, 2 on disk
    match status_kind {
        0 => (Color::SUCCESS, "●"),
        1 => (Color::WARNING, "○"),
        _ => (Color::DANGER, "!"),
    }
}
