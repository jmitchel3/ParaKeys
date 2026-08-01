//! ParaKeys GUI design system: tokens, theme, and reusable controls.
//!
//! One source of truth for spacing, type, color, and chrome so the shell stays
//! consistent and easy to evolve without hunting magic numbers.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Sense, Stroke, Ui, Vec2,
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
    pub const PILL: u8 = 100;
}

#[derive(Clone, Copy)]
pub struct Type;
impl Type {
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 13.0;
    pub const CALL_OUT: f32 = 14.0;
    pub const TITLE: f32 = 15.0;
    pub const HEADLINE: f32 = 22.0;
    pub const HERO: f32 = 26.0;
}

// ─── Semantic color ──────────────────────────────────────────────────────────

/// Light, system-adjacent palette. Prefer semantic names over raw hex at call sites.
#[derive(Clone, Copy)]
pub struct Color;
impl Color {
    pub const BG: Color32 = Color32::from_rgb(242, 242, 247);
    pub const BG_SIDE: Color32 = Color32::from_rgb(246, 246, 248);
    pub const SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const SURFACE_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);
    pub const FILL: Color32 = Color32::from_rgb(232, 232, 237);
    pub const FILL_HOVER: Color32 = Color32::from_rgb(220, 220, 225);
    pub const BORDER: Color32 = Color32::from_rgb(220, 220, 225);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(232, 232, 235);
    pub const TEXT: Color32 = Color32::from_rgb(28, 28, 30);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(110, 110, 115);
    pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(142, 142, 147);
    pub const TEXT_ON_ACCENT: Color32 = Color32::WHITE;
    pub const ACCENT: Color32 = Color32::from_rgb(0, 122, 255);
    pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(0, 100, 220);
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 199, 89);
    pub const WARNING: Color32 = Color32::from_rgb(255, 149, 0);
    pub const DANGER: Color32 = Color32::from_rgb(255, 59, 48);
    pub const SELECTION: Color32 = Color32::from_rgb(0, 122, 255);
    pub const WARNING_BG: Color32 = Color32::from_rgb(255, 250, 240);
    pub const WARNING_BORDER: Color32 = Color32::from_rgb(230, 200, 160);
}

// ─── Layout constants ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Layout;
impl Layout {
    pub const SIDEBAR_W: f32 = 232.0;
    pub const LIST_W: f32 = 288.0;
    pub const TOOLBAR_H: f32 = 52.0;
    pub const TILE: Vec2 = Vec2::new(100.0, 76.0);
    pub const ROW_MIN_H: f32 = 52.0;
    pub const CONTROL_H: f32 = 28.0;
    pub const ICON_SM: f32 = 22.0;
    pub const ICON_MD: f32 = 32.0;
    pub const ICON_LG: f32 = 72.0;
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
    v.selection.bg_fill = Color::SELECTION;
    v.selection.stroke = Stroke::new(1.0, Color::ACCENT);
    v.extreme_bg_color = Color::SURFACE;
    v.faint_bg_color = Color::FILL;
    v.window_corner_radius = CornerRadius::same(0);
    v.menu_corner_radius = CornerRadius::same(Radius::MD);
    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(Space::SM, Space::SM);
    style.spacing.button_padding = Vec2::new(Space::MD, 6.0);
    style.spacing.indent = Space::LG;
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}

// ─── Typography helpers ──────────────────────────────────────────────────────

pub fn text_hero(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::HERO).strong().color(Color::TEXT)
}
pub fn text_headline(s: impl Into<String>) -> RichText {
    RichText::new(s).size(Type::HEADLINE).strong().color(Color::TEXT)
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
    RichText::new(s).size(size).monospace().color(Color::TEXT_SECONDARY)
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
        .fill(Color::BG)
        .stroke(Stroke::new(1.0, Color::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(Space::LG as i8, Space::SM as i8 + 2))
}

pub fn card() -> Frame {
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
        .inner_margin(Margin::same(Space::MD as i8))
}

// ─── Controls ────────────────────────────────────────────────────────────────

pub fn primary_button(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .size(Type::BODY)
                .color(Color::TEXT_ON_ACCENT)
                .strong(),
        )
        .fill(Color::ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(Radius::SM))
        .min_size(Vec2::new(0.0, Layout::CONTROL_H)),
    )
}

pub fn secondary_button(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(Type::BODY).color(Color::TEXT))
            .fill(Color::FILL)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(Radius::SM))
            .min_size(Vec2::new(0.0, Layout::CONTROL_H)),
    )
}

pub fn toolbar_pill(ui: &mut Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(Type::BODY).color(Color::TEXT))
            .fill(Color::SURFACE)
            .stroke(Stroke::new(1.0, Color::BORDER))
            .corner_radius(CornerRadius::same(Radius::PILL))
            .min_size(Vec2::new(Layout::CONTROL_H, Layout::CONTROL_H)),
    )
}

pub fn search_field(ui: &mut Ui, text: &mut String, width: f32) -> egui::Response {
    let mut resp = None;
    card()
        .corner_radius(CornerRadius::same(Radius::PILL))
        .inner_margin(Margin::symmetric(Space::MD as i8, 5))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌕").size(Type::BODY).color(Color::TEXT_TERTIARY));
                resp = Some(
                    ui.add(
                        egui::TextEdit::singleline(text)
                            .hint_text("Search")
                            .frame(false)
                            .desired_width(width - 36.0)
                            .font(FontId::proportional(Type::BODY)),
                    ),
                );
            });
        });
    resp.unwrap()
}

pub fn section_label(ui: &mut Ui, label: &str) {
    ui.add_space(Space::SM);
    ui.label(text_section(label));
    ui.add_space(Space::XS);
}

/// Soft category tile (Passwords-style).
pub fn category_tile(
    ui: &mut Ui,
    title: &str,
    count: usize,
    glyph: &str,
    accent: Color32,
    selected: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Layout::TILE, Sense::click());
    let hovered = resp.hovered();
    let bg = if selected {
        accent
    } else if hovered {
        Color::FILL_HOVER
    } else {
        Color::SURFACE
    };
    let stroke = if selected {
        Stroke::NONE
    } else {
        Stroke::new(1.0, Color::BORDER)
    };
    ui.painter()
        .rect(rect, CornerRadius::same(Radius::LG), bg, stroke, egui::StrokeKind::Inside);

    let fg = if selected {
        Color::TEXT_ON_ACCENT
    } else {
        Color::TEXT
    };
    let muted = if selected {
        Color32::from_rgba_unmultiplied(255, 255, 255, 200)
    } else {
        Color::TEXT_SECONDARY
    };

    let icon_c = rect.left_top() + Vec2::new(20.0, 24.0);
    let icon_bg = if selected {
        Color32::from_rgba_unmultiplied(255, 255, 255, 40)
    } else {
        accent.gamma_multiply(0.16)
    };
    ui.painter().circle_filled(icon_c, 11.0, icon_bg);
    ui.painter().text(
        icon_c,
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(11.0),
        if selected {
            Color::TEXT_ON_ACCENT
        } else {
            accent
        },
    );

    ui.painter().text(
        rect.right_top() + Vec2::new(-12.0, 12.0),
        egui::Align2::RIGHT_TOP,
        format!("{count}"),
        FontId::proportional(Type::BODY),
        muted,
    );
    ui.painter().text(
        rect.left_bottom() + Vec2::new(14.0, -12.0),
        egui::Align2::LEFT_BOTTOM,
        title,
        FontId::proportional(Type::BODY),
        fg,
    );
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
    let fill = if selected {
        Color::SELECTION
    } else {
        Color32::TRANSPARENT
    };
    let out = Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(Radius::SM))
        .inner_margin(Margin::symmetric(Space::MD as i8, Space::SM as i8 + 2))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(Layout::ROW_MIN_H - 8.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(Layout::ICON_MD), Sense::hover());
                let icon_bg = if selected {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 40)
                } else {
                    accent.gamma_multiply(0.18)
                };
                ui.painter().circle_filled(r.center(), 16.0, icon_bg);
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    glyph,
                    FontId::proportional(12.0),
                    if selected {
                        Color::TEXT_ON_ACCENT
                    } else {
                        accent
                    },
                );
                ui.add_space(Space::SM);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title)
                            .size(Type::CALL_OUT)
                            .strong()
                            .color(if selected {
                                Color::TEXT_ON_ACCENT
                            } else {
                                Color::TEXT
                            }),
                    );
                    ui.label(RichText::new(subtitle).size(Type::CAPTION).color(
                        if selected {
                            Color32::from_rgba_unmultiplied(255, 255, 255, 200)
                        } else {
                            Color::TEXT_SECONDARY
                        },
                    ));
                });
            });
        });
    ui.interact(out.response.rect, out.response.id, Sense::click())
}

pub fn detail_field(ui: &mut Ui, label: &str, value: &str, mono: bool) {
    ui.horizontal(|ui| {
        ui.set_min_height(40.0);
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
            ui.add_space((h * 0.18).clamp(Space::XXL, 100.0));
            // Soft rounded badge (no emoji dependency for primary empty)
            let (r, _) = ui.allocate_exact_size(Vec2::splat(Layout::ICON_LG), Sense::hover());
            ui.painter().rect(
                r,
                CornerRadius::same(Radius::XL),
                Color::ACCENT,
                Stroke::NONE,
                egui::StrokeKind::Inside,
            );
            // Simple key shape via text for now (vector SF Symbols need native)
            ui.painter().text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                "key",
                FontId::proportional(14.0),
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
