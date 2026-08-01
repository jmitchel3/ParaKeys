//! ParaKeys GUI — three-pane shell modeled on Apple Passwords (categories · list · detail).

use std::path::PathBuf;
use std::process::Command;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, RichText, Sense, Stroke,
    Vec2,
};
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, KeyStatus};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

// ─── Tokens from Passwords-like light UI ───────────────────────────────────
struct P;
impl P {
    const BG: Color32 = Color32::from_rgb(242, 242, 247);
    const SIDE: Color32 = Color32::from_rgb(246, 246, 248);
    const CARD: Color32 = Color32::from_rgb(255, 255, 255);
    const HAIR: Color32 = Color32::from_rgb(220, 220, 225);
    const TEXT: Color32 = Color32::from_rgb(28, 28, 30);
    const SEC: Color32 = Color32::from_rgb(110, 110, 115);
    const TERT: Color32 = Color32::from_rgb(142, 142, 147);
    const BLUE: Color32 = Color32::from_rgb(0, 122, 255);
    const BLUE_SEL: Color32 = Color32::from_rgb(0, 122, 255);
    const CHIP: Color32 = Color32::from_rgb(232, 232, 237);
    const GREEN: Color32 = Color32::from_rgb(52, 199, 89);
    const ORANGE: Color32 = Color32::from_rgb(255, 149, 0);
    const RED: Color32 = Color32::from_rgb(255, 59, 48);
    const PURPLE: Color32 = Color32::from_rgb(175, 82, 222);
    const YELLOW: Color32 = Color32::from_rgb(255, 204, 0);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    All,
    Set,
    Missing,
    OnDisk,
    Projects,
}

impl Category {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Set => "Set",
            Self::Missing => "Missing",
            Self::OnDisk => "On Disk",
            Self::Projects => "Project",
        }
    }
    fn color(self) -> Color32 {
        match self {
            Self::All => P::BLUE,
            Self::Set => P::GREEN,
            Self::Missing => P::ORANGE,
            Self::OnDisk => P::RED,
            Self::Projects => P::PURPLE,
        }
    }
    fn glyph(self) -> &'static str {
        match self {
            Self::All => "🔑",
            Self::Set => "✓",
            Self::Missing => "?",
            Self::OnDisk => "!",
            Self::Projects => "⌘",
        }
    }
}

fn theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill = P::BG;
    v.window_fill = P::BG;
    v.override_text_color = Some(P::TEXT);
    v.widgets.inactive.bg_fill = P::CHIP;
    v.widgets.hovered.bg_fill = Color32::from_rgb(220, 220, 225);
    v.widgets.active.bg_fill = P::BLUE.gamma_multiply(0.85);
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.selection.bg_fill = P::BLUE_SEL;
    v.extreme_bg_color = P::CARD;
    ctx.set_visuals(v);
    let mut s = (*ctx.style()).clone();
    s.spacing.item_spacing = Vec2::new(8.0, 6.0);
    s.spacing.button_padding = Vec2::new(10.0, 5.0);
    s.interaction.selectable_labels = false;
    ctx.set_style(s);
}

fn pill(ui: &mut egui::Ui, text: &str, filled: bool) -> egui::Response {
    if filled {
        ui.add(
            egui::Button::new(RichText::new(text).color(Color32::WHITE).size(13.0))
                .fill(P::BLUE)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(16))
                .min_size(Vec2::new(36.0, 28.0)),
        )
    } else {
        ui.add(
            egui::Button::new(RichText::new(text).color(P::TEXT).size(13.0))
                .fill(P::CHIP)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(16))
                .min_size(Vec2::new(36.0, 28.0)),
        )
    }
}

fn category_tile(
    ui: &mut egui::Ui,
    cat: Category,
    count: usize,
    selected: bool,
) -> egui::Response {
    let size = Vec2::new(96.0, 72.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let bg = if selected { cat.color() } else { P::CARD };
    let fg = if selected { Color32::WHITE } else { P::TEXT };
    let sub = if selected {
        Color32::from_rgba_unmultiplied(255, 255, 255, 200)
    } else {
        P::SEC
    };
    let stroke = if selected {
        Stroke::NONE
    } else {
        Stroke::new(1.0, P::HAIR)
    };

    ui.painter()
        .rect(rect, CornerRadius::same(14), bg, stroke, egui::StrokeKind::Inside);

    // icon circle
    let icon_c = Pos2::new(rect.left() + 22.0, rect.top() + 26.0);
    let icon_bg = if selected {
        Color32::from_rgba_unmultiplied(255, 255, 255, 40)
    } else {
        cat.color().gamma_multiply(0.18)
    };
    ui.painter().circle_filled(icon_c, 12.0, icon_bg);
    ui.painter().text(
        icon_c,
        Align2::CENTER_CENTER,
        cat.glyph(),
        FontId::proportional(11.0),
        if selected { Color32::WHITE } else { cat.color() },
    );

    // count top-right
    ui.painter().text(
        Pos2::new(rect.right() - 12.0, rect.top() + 14.0),
        Align2::RIGHT_TOP,
        format!("{count}"),
        FontId::proportional(13.0),
        if selected {
            Color32::from_rgba_unmultiplied(255, 255, 255, 220)
        } else {
            P::SEC
        },
    );

    // label
    ui.painter().text(
        Pos2::new(rect.left() + 14.0, rect.bottom() - 14.0),
        Align2::LEFT_BOTTOM,
        cat.label(),
        FontId::proportional(13.0),
        fg,
    );

    let _ = sub;
    resp
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 640.0])
            .with_min_inner_size([860.0, 520.0])
            .with_title("ParaKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "ParaKeys",
        options,
        Box::new(|cc| {
            theme(&cc.egui_ctx);
            let mut app = App::default();
            if let Ok(cwd) = project_root(None) {
                app.path = cwd.display().to_string();
                app.reload();
            }
            Ok(Box::new(app))
        }),
    )
}

struct KeyRow {
    name: String,
    status: KeyStatus,
    value: Option<String>,
}

struct App {
    path: String,
    status_msg: String,
    keys: Vec<KeyRow>,
    category: Category,
    selected: Option<usize>,
    reveal: bool,
    prev_reveal: bool,
    recovery: String,
    run_cmd: String,
    backend: String,
    env_name: String,
    has_vault: bool,
    has_unlock: bool,
    search: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: String::new(),
            status_msg: String::new(),
            keys: Vec::new(),
            category: Category::All,
            selected: None,
            reveal: false,
            prev_reveal: false,
            recovery: String::new(),
            run_cmd: String::new(),
            backend: String::new(),
            env_name: String::new(),
            has_vault: false,
            has_unlock: false,
            search: String::new(),
        }
    }
}

impl App {
    fn root(&self) -> Result<PathBuf, String> {
        let p = self.path.trim();
        if p.is_empty() {
            project_root(None).map_err(|e| e.to_string())
        } else {
            Ok(PathBuf::from(p))
        }
    }

    fn project_name(&self) -> String {
        self.root()
            .ok()
            .and_then(|r| r.file_name().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "Project".into())
    }

    fn short_path(&self) -> String {
        let p = self.path.trim();
        if p.is_empty() {
            return String::new();
        }
        if let Ok(home) = std::env::var("HOME") {
            if let Some(rest) = p.strip_prefix(&home) {
                return format!("~{rest}");
            }
        }
        p.to_string()
    }

    fn counts(&self) -> (usize, usize, usize, usize) {
        let all = self.keys.len();
        let set = self
            .keys
            .iter()
            .filter(|k| k.status == KeyStatus::SetInVault)
            .count();
        let missing = self
            .keys
            .iter()
            .filter(|k| k.status == KeyStatus::NotSet)
            .count();
        let on_disk = self
            .keys
            .iter()
            .filter(|k| k.status == KeyStatus::PlaintextOnDisk)
            .count();
        (all, set, missing, on_disk)
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let q = self.search.to_ascii_lowercase();
        self.keys
            .iter()
            .enumerate()
            .filter(|(_, k)| {
                let cat_ok = match self.category {
                    Category::All | Category::Projects => true,
                    Category::Set => k.status == KeyStatus::SetInVault,
                    Category::Missing => k.status == KeyStatus::NotSet,
                    Category::OnDisk => k.status == KeyStatus::PlaintextOnDisk,
                };
                let search_ok =
                    q.is_empty() || k.name.to_ascii_lowercase().contains(&q);
                cat_ok && search_ok
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn reload(&mut self) {
        let prev_name = self
            .selected
            .and_then(|i| self.keys.get(i).map(|k| k.name.clone()));
        self.keys.clear();
        self.backend.clear();
        self.env_name.clear();
        self.has_vault = false;
        self.has_unlock = false;
        self.status_msg.clear();

        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status_msg = e;
                return;
            }
        };

        if let Some(b) = detect_backend(&root) {
            self.backend = b.as_str().to_string();
        }
        self.has_vault = default_vault_path(&root).is_file();
        self.has_unlock = has_unlock_key(&root);
        if !self.has_vault || !self.has_unlock {
            self.selected = None;
            return;
        }

        let key = match load_unlock_key(&root) {
            Ok(k) => k,
            Err(e) => {
                self.status_msg = e.to_string();
                return;
            }
        };
        let vault = match load_vault(&root, &key) {
            Ok(v) => v,
            Err(e) => {
                self.status_msg = e.to_string();
                return;
            }
        };

        let mut env_map = std::collections::BTreeMap::new();
        if root.join(".env").is_file() {
            if let Ok(env) = load_env_file(&root.join(".env")) {
                for (k, v) in env.assignments() {
                    env_map.insert(k.to_string(), v.to_string());
                }
            }
        }

        let mut names: std::collections::BTreeSet<_> = vault.keys.keys().cloned().collect();
        names.extend(env_map.keys().cloned());
        for name in names {
            let st = classify_key(&vault, &name, env_map.get(&name).map(String::as_str));
            let value = if self.reveal {
                vault.get(&name).map(|s| s.to_string())
            } else {
                None
            };
            self.keys.push(KeyRow {
                name,
                status: st,
                value,
            });
        }

        self.env_name = load_config(&root)
            .map(|c| c.env_name)
            .unwrap_or_else(|_| "local".into());

        // restore selection
        self.selected = prev_name.and_then(|n| self.keys.iter().position(|k| k.name == n));
        if self.selected.is_none() && !self.keys.is_empty() {
            self.selected = Some(0);
        }
    }

    fn init_vault(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status_msg = e;
                return;
            }
        };
        if default_vault_path(&root).is_file() {
            self.status_msg = "Vault already exists.".into();
            return;
        }
        let key = VaultKey::generate();
        if let Err(e) = save_vault(&root, &VaultData::new(), &key) {
            self.status_msg = e.to_string();
            return;
        }
        match store_unlock_key(&root, &key) {
            Ok(outcome) => {
                self.recovery = encode_recovery_code(&key);
                let mut msg = match outcome.backend {
                    WalletBackend::KeychainUserPresence => {
                        "Vault created. Keychain + Touch ID when available.".into()
                    }
                    WalletBackend::Keychain => "Vault created. Unlock key in Keychain.".to_string(),
                    WalletBackend::File => "Vault created. Unlock key in file wallet.".to_string(),
                };
                for n in &outcome.notes {
                    msg.push('\n');
                    msg.push_str(n);
                }
                self.status_msg = msg;
                self.reload();
            }
            Err(e) => self.status_msg = e.to_string(),
        }
    }

    fn import_env(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status_msg = e;
                return;
            }
        };
        let env = root.join(".env");
        let out = Command::new(cli())
            .args([
                "import",
                "--path",
                &root.display().to_string(),
                env.to_str().unwrap_or(".env"),
            ])
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                self.status_msg = if s.is_empty() {
                    "Imported.".into()
                } else {
                    s
                };
                self.reload();
            }
            Ok(o) => self.status_msg = String::from_utf8_lossy(&o.stderr).trim().to_string(),
            Err(e) => self.status_msg = e.to_string(),
        }
    }

    fn run_cmd(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status_msg = e;
                return;
            }
        };
        let cmd = self.run_cmd.trim();
        if cmd.is_empty() {
            self.status_msg = "Enter a command.".into();
            return;
        }
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut args = vec![
            "run".into(),
            "--path".into(),
            root.display().to_string(),
            "--".into(),
        ];
        args.extend(parts.iter().map(|s| (*s).to_string()));
        match Command::new(cli()).args(&args).output() {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout);
                let err = String::from_utf8_lossy(&o.stderr);
                self.status_msg = if o.status.success() {
                    if out.trim().is_empty() {
                        "Done.".into()
                    } else {
                        out.trim().to_string()
                    }
                } else if !err.trim().is_empty() {
                    err.trim().to_string()
                } else {
                    out.trim().to_string()
                };
            }
            Err(e) => self.status_msg = e.to_string(),
        }
    }
}

fn cli() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            let mut c = p;
            c.set_file_name("parakeys");
            c.is_file().then_some(c)
        })
        .unwrap_or_else(|| PathBuf::from("parakeys"))
}

fn status_label_short(s: KeyStatus) -> &'static str {
    match s {
        KeyStatus::SetInVault => "Set in vault",
        KeyStatus::NotSet => "Not set",
        KeyStatus::PlaintextOnDisk => "Still in .env file",
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme(ctx);
        let (n_all, n_set, n_miss, n_disk) = self.counts();

        // ── Toolbar (Passwords-style floating controls) ───────────────────
        egui::TopBottomPanel::top("toolbar")
            .exact_height(54.0)
            .frame(
                Frame::new()
                    .fill(P::BG)
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Title block
                    ui.vertical(|ui| {
                        let title = match self.category {
                            Category::All => "All",
                            c => c.label(),
                        };
                        ui.label(RichText::new(title).size(15.0).strong().color(P::TEXT));
                        let count = match self.category {
                            Category::All | Category::Projects => n_all,
                            Category::Set => n_set,
                            Category::Missing => n_miss,
                            Category::OnDisk => n_disk,
                        };
                        ui.label(
                            RichText::new(format!(
                                "{count} Item{}",
                                if count == 1 { "" } else { "s" }
                            ))
                            .size(11.0)
                            .color(P::SEC),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Search pill
                        Frame::new()
                            .fill(P::CARD)
                            .stroke(Stroke::new(1.0, P::HAIR))
                            .corner_radius(CornerRadius::same(16))
                            .inner_margin(Margin::symmetric(12, 5))
                            .show(ui, |ui| {
                                ui.set_width(160.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("⌕").size(13.0).color(P::TERT));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.search)
                                            .hint_text("Search")
                                            .frame(false)
                                            .desired_width(130.0),
                                    );
                                });
                            });

                        ui.add_space(6.0);
                        if pill(ui, "  Edit  ", false).clicked() {
                            // focus detail / no-op for now
                        }
                        if pill(ui, "  +  ", false).clicked() {
                            self.import_env();
                        }
                        if pill(ui, "  Open…  ", false).clicked() {
                            if let Some(f) = rfd::FileDialog::new().pick_folder() {
                                self.path = f.display().to_string();
                                self.reload();
                            }
                        }
                    });
                });
            });

        // ── Left: category tiles + project (like Passwords sidebar) ────────
        egui::SidePanel::left("cats")
            .exact_width(228.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(P::SIDE)
                    .inner_margin(Margin::symmetric(14, 12)),
            )
            .show(ctx, |ui| {
                // 2×2 tile grid
                ui.horizontal(|ui| {
                    if category_tile(ui, Category::All, n_all, self.category == Category::All)
                        .clicked()
                    {
                        self.category = Category::All;
                    }
                    ui.add_space(8.0);
                    if category_tile(ui, Category::Set, n_set, self.category == Category::Set)
                        .clicked()
                    {
                        self.category = Category::Set;
                    }
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if category_tile(
                        ui,
                        Category::Missing,
                        n_miss,
                        self.category == Category::Missing,
                    )
                    .clicked()
                    {
                        self.category = Category::Missing;
                    }
                    ui.add_space(8.0);
                    if category_tile(
                        ui,
                        Category::OnDisk,
                        n_disk,
                        self.category == Category::OnDisk,
                    )
                    .clicked()
                    {
                        self.category = Category::OnDisk;
                    }
                });

                ui.add_space(18.0);
                ui.label(
                    RichText::new("Project")
                        .size(11.0)
                        .color(P::TERT)
                        .strong(),
                );
                ui.add_space(6.0);

                let proj_sel = self.category == Category::Projects;
                let out = Frame::new()
                    .fill(if proj_sel { P::BLUE_SEL } else { Color32::TRANSPARENT })
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
                            ui.painter().rect_filled(
                                r,
                                CornerRadius::same(5),
                                if proj_sel {
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 40)
                                } else {
                                    P::YELLOW.gamma_multiply(0.35)
                                },
                            );
                            ui.painter().text(
                                r.center(),
                                Align2::CENTER_CENTER,
                                "📁",
                                FontId::proportional(11.0),
                                P::TEXT,
                            );
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(self.project_name())
                                        .size(13.0)
                                        .color(if proj_sel {
                                            Color32::WHITE
                                        } else {
                                            P::TEXT
                                        })
                                        .strong(),
                                );
                                let w = if self.backend.is_empty() {
                                    if self.has_vault {
                                        "locked"
                                    } else {
                                        "no vault"
                                    }
                                } else {
                                    &self.backend
                                };
                                ui.label(
                                    RichText::new(w).size(11.0).color(if proj_sel {
                                        Color32::from_rgba_unmultiplied(255, 255, 255, 190)
                                    } else {
                                        P::SEC
                                    }),
                                );
                            });
                        });
                    });
                if ui
                    .interact(out.response.rect, out.response.id, Sense::click())
                    .clicked()
                {
                    self.category = Category::Projects;
                }

                ui.add_space(4.0);
                ui.label(RichText::new(self.short_path()).size(10.0).color(P::TERT));

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(4.0);
                    let r = ui.checkbox(
                        &mut self.reveal,
                        RichText::new("Show Values").size(12.0).color(P::SEC),
                    );
                    if r.changed() || self.reveal != self.prev_reveal {
                        self.prev_reveal = self.reveal;
                        self.reload();
                    }
                });
            });

        // ── Middle: list ──────────────────────────────────────────────────
        egui::SidePanel::left("list")
            .exact_width(280.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(P::CARD)
                    .stroke(Stroke::new(1.0, P::HAIR))
                    .inner_margin(Margin::ZERO),
            )
            .show(ctx, |ui| {
                if !self.has_vault {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(RichText::new("No items").size(13.0).color(P::SEC));
                    });
                    return;
                }
                if !self.has_unlock {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(RichText::new("Locked").size(13.0).color(P::SEC));
                    });
                    return;
                }

                let indices = self.filtered_indices();
                if indices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(RichText::new("No items").size(13.0).color(P::SEC));
                    });
                    return;
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (list_i, &ki) in indices.iter().enumerate() {
                            let row = &self.keys[ki];
                            let selected = self.selected == Some(ki);
                            let fill = if selected {
                                P::BLUE_SEL
                            } else {
                                Color32::TRANSPARENT
                            };
                            let fg = if selected { Color32::WHITE } else { P::TEXT };
                            let sub = if selected {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 200)
                            } else {
                                P::SEC
                            };

                            let out = Frame::new()
                                .fill(fill)
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(12, 10))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let (r, _) =
                                            ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                                        let c = match row.status {
                                            KeyStatus::SetInVault => P::GREEN,
                                            KeyStatus::NotSet => P::ORANGE,
                                            KeyStatus::PlaintextOnDisk => P::RED,
                                        };
                                        ui.painter().circle_filled(
                                            r.center(),
                                            16.0,
                                            if selected {
                                                Color32::from_rgba_unmultiplied(255, 255, 255, 35)
                                            } else {
                                                c.gamma_multiply(0.2)
                                            },
                                        );
                                        ui.painter().text(
                                            r.center(),
                                            Align2::CENTER_CENTER,
                                            match row.status {
                                                KeyStatus::SetInVault => "●",
                                                KeyStatus::NotSet => "○",
                                                KeyStatus::PlaintextOnDisk => "!",
                                            },
                                            FontId::proportional(12.0),
                                            if selected { Color32::WHITE } else { c },
                                        );
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&row.name)
                                                    .size(14.0)
                                                    .color(fg)
                                                    .strong(),
                                            );
                                            ui.label(
                                                RichText::new(status_label_short(row.status))
                                                    .size(11.0)
                                                    .color(sub),
                                            );
                                        });
                                    });
                                });
                            if ui
                                .interact(out.response.rect, out.response.id, Sense::click())
                                .clicked()
                            {
                                self.selected = Some(ki);
                            }
                            if list_i + 1 < indices.len() && !selected {
                                let y = ui.cursor().top();
                                let rect = ui.max_rect();
                                ui.painter().hline(
                                    (rect.left() + 12.0)..=(rect.right() - 12.0),
                                    y,
                                    Stroke::new(1.0, P::HAIR),
                                );
                            }
                        }
                    });
            });

        // ── Right: detail ─────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(P::BG)
                    .inner_margin(Margin::symmetric(28, 24)),
            )
            .show(ctx, |ui| {
                if !self.recovery.is_empty() {
                    Frame::new()
                        .fill(Color32::from_rgb(255, 250, 240))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(230, 200, 160)))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Recovery code")
                                    .size(12.0)
                                    .strong()
                                    .color(P::ORANGE),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(&self.recovery)
                                    .size(14.0)
                                    .monospace()
                                    .color(P::TEXT),
                            );
                            ui.label(
                                RichText::new("Store offline. Shown once.")
                                    .size(11.0)
                                    .color(P::SEC),
                            );
                        });
                    ui.add_space(16.0);
                }

                if !self.status_msg.is_empty() {
                    ui.label(RichText::new(&self.status_msg).size(12.0).color(P::SEC));
                    ui.add_space(10.0);
                }

                if !self.has_vault {
                    // Passwords-empty style: big icon + title + primary
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.18);
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(72.0), Sense::hover());
                        ui.painter().rect(
                            r,
                            CornerRadius::same(18),
                            P::BLUE,
                            Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            "🔑",
                            FontId::proportional(28.0),
                            Color32::WHITE,
                        );
                        ui.add_space(18.0);
                        ui.label(
                            RichText::new("No Vault")
                                .size(26.0)
                                .strong()
                                .color(P::TEXT),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Create a vault for this project to store\nenvironment secrets securely.",
                            )
                            .size(14.0)
                            .color(P::SEC),
                        );
                        ui.add_space(22.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("  Create Vault  ")
                                        .color(Color32::WHITE)
                                        .size(14.0),
                                )
                                .fill(P::BLUE)
                                .stroke(Stroke::NONE)
                                .corner_radius(CornerRadius::same(10))
                                .min_size(Vec2::new(140.0, 36.0)),
                            )
                            .clicked()
                        {
                            self.init_vault();
                        }
                    });
                    return;
                }

                if !self.has_unlock {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(RichText::new("Locked").size(24.0).strong().color(P::TEXT));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("parakeys init --recover '<code>'")
                                .size(13.0)
                                .monospace()
                                .color(P::SEC),
                        );
                    });
                    return;
                }

                if self.keys.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(90.0);
                        ui.label(RichText::new("No Keys").size(24.0).strong().color(P::TEXT));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Import a .env to move secrets into the vault.")
                                .size(14.0)
                                .color(P::SEC),
                        );
                        ui.add_space(18.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("  Import .env  ")
                                        .color(Color32::WHITE)
                                        .size(14.0),
                                )
                                .fill(P::BLUE)
                                .stroke(Stroke::NONE)
                                .corner_radius(CornerRadius::same(10))
                                .min_size(Vec2::new(130.0, 34.0)),
                            )
                            .clicked()
                        {
                            self.import_env();
                        }
                    });
                    return;
                }

                // Detail of selected key
                let idx = self.selected.or(self.filtered_indices().first().copied());
                let Some(ki) = idx else {
                    return;
                };
                let Some(row) = self.keys.get(ki) else {
                    return;
                };

                // Hero icon
                ui.vertical_centered(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(80.0), Sense::hover());
                    let c = match row.status {
                        KeyStatus::SetInVault => P::GREEN,
                        KeyStatus::NotSet => P::ORANGE,
                        KeyStatus::PlaintextOnDisk => P::RED,
                    };
                    ui.painter().rect(
                        r,
                        CornerRadius::same(20),
                        c,
                        Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        "🔑",
                        FontId::proportional(32.0),
                        Color32::WHITE,
                    );
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(&row.name)
                            .size(26.0)
                            .strong()
                            .color(P::TEXT),
                    );
                    ui.label(
                        RichText::new(status_label_short(row.status))
                            .size(13.0)
                            .color(P::SEC),
                    );
                });

                ui.add_space(24.0);

                // Detail fields card (Passwords field list)
                Frame::new()
                    .fill(P::CARD)
                    .stroke(Stroke::new(1.0, P::HAIR))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(18, 6))
                    .show(ui, |ui| {
                        detail_row(ui, "Name", &row.name, false);
                        hairline(ui);
                        let val_display = if self.reveal {
                            row.value
                                .clone()
                                .unwrap_or_else(|| "—".into())
                        } else if row.status == KeyStatus::SetInVault {
                            "••••••••••••".into()
                        } else {
                            "—".into()
                        };
                        detail_row(ui, "Value", &val_display, self.reveal);
                        hairline(ui);
                        detail_row(ui, "Status", status_label_short(row.status), false);
                        hairline(ui);
                        detail_row(
                            ui,
                            "Environment",
                            if self.env_name.is_empty() {
                                "—"
                            } else {
                                &self.env_name
                            },
                            false,
                        );
                        hairline(ui);
                        detail_row(
                            ui,
                            "Wallet",
                            if self.backend.is_empty() {
                                "—"
                            } else {
                                &self.backend
                            },
                            false,
                        );
                        hairline(ui);
                        detail_row(ui, "Project", &self.project_name(), false);
                    });

                ui.add_space(16.0);
                // Run with secrets
                Frame::new()
                    .fill(P::CARD)
                    .stroke(Stroke::new(1.0, P::HAIR))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Run").size(12.0).color(P::TERT));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.run_cmd)
                                    .hint_text("npm start")
                                    .desired_width(ui.available_width() - 70.0),
                            );
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Run").color(Color32::WHITE).size(13.0),
                                    )
                                    .fill(P::BLUE)
                                    .corner_radius(CornerRadius::same(8)),
                                )
                                .clicked()
                            {
                                self.run_cmd();
                            }
                        });
                    });
            });
    }
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str, mono: bool) {
    ui.horizontal(|ui| {
        ui.set_min_height(40.0);
        ui.label(RichText::new(label).size(13.0).color(P::TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut t = RichText::new(value).size(13.0).color(P::SEC);
            if mono {
                t = t.monospace();
            }
            ui.label(t);
        });
    });
}

fn hairline(ui: &mut egui::Ui) {
    let y = ui.cursor().top();
    let r = ui.max_rect();
    ui.painter()
        .hline(r.x_range(), y, Stroke::new(1.0, P::HAIR));
    ui.add_space(1.0);
}
