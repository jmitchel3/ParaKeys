//! ParaKeys GUI — light, single-column Mac utility. No sidebar theater.

use std::path::PathBuf;
use std::process::Command;

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Vec2,
};
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, KeyStatus};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

/// Light, system-adjacent palette (macOS window chrome feel).
struct C;
impl C {
    const WINDOW: Color32 = Color32::from_rgb(246, 246, 246);
    const SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    const HAIRLINE: Color32 = Color32::from_rgb(220, 220, 222);
    const LABEL: Color32 = Color32::from_rgb(29, 29, 31);
    const SECONDARY: Color32 = Color32::from_rgb(110, 110, 115);
    const TERTIARY: Color32 = Color32::from_rgb(142, 142, 147);
    const BLUE: Color32 = Color32::from_rgb(0, 122, 255);
    const BLUE_PRESS: Color32 = Color32::from_rgb(0, 100, 220);
    const FILL_CONTROL: Color32 = Color32::from_rgb(232, 232, 234);
    const GREEN: Color32 = Color32::from_rgb(40, 160, 70);
    const ORANGE: Color32 = Color32::from_rgb(200, 120, 0);
    const ROW_SEP: Color32 = Color32::from_rgb(235, 235, 237);
}

fn theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill = C::WINDOW;
    v.window_fill = C::WINDOW;
    v.override_text_color = Some(C::LABEL);
    v.widgets.inactive.bg_fill = C::FILL_CONTROL;
    v.widgets.hovered.bg_fill = Color32::from_rgb(220, 220, 224);
    v.widgets.active.bg_fill = C::BLUE_PRESS;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, C::HAIRLINE);
    v.extreme_bg_color = C::SURFACE;
    v.faint_bg_color = C::FILL_CONTROL;
    v.selection.bg_fill = Color32::from_rgb(0, 122, 255).gamma_multiply(0.15);
    v.window_corner_radius = CornerRadius::same(0);
    ctx.set_visuals(v);

    let mut s = (*ctx.style()).clone();
    s.spacing.item_spacing = Vec2::new(8.0, 6.0);
    s.spacing.button_padding = Vec2::new(11.0, 5.0);
    s.interaction.selectable_labels = false;
    ctx.set_style(s);
}

fn btn_primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(Color32::WHITE).size(13.0))
            .fill(C::BLUE)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(5))
            .min_size(Vec2::new(100.0, 26.0)),
    )
}

fn btn_secondary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).color(C::LABEL).size(13.0))
            .fill(C::FILL_CONTROL)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(5))
            .min_size(Vec2::new(0.0, 26.0)),
    )
}

fn chip(ui: &mut egui::Ui, status: KeyStatus) {
    let (t, c) = match status {
        KeyStatus::SetInVault => ("Set", C::GREEN),
        KeyStatus::NotSet => ("Not Set", C::TERTIARY),
        KeyStatus::PlaintextOnDisk => ("In File", C::ORANGE),
    };
    ui.label(RichText::new(t).size(12.0).color(c));
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([560.0, 400.0])
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

struct App {
    path: String,
    status: String,
    keys: Vec<(String, KeyStatus)>,
    revealed: Vec<(String, String)>,
    reveal: bool,
    prev_reveal: bool,
    recovery: String,
    run_cmd: String,
    backend: String,
    env_name: String,
    has_vault: bool,
    has_unlock: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: String::new(),
            status: String::new(),
            keys: Vec::new(),
            revealed: Vec::new(),
            reveal: false,
            prev_reveal: false,
            recovery: String::new(),
            run_cmd: String::new(),
            backend: String::new(),
            env_name: String::new(),
            has_vault: false,
            has_unlock: false,
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

    fn title(&self) -> String {
        self.root()
            .ok()
            .and_then(|r| r.file_name().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "ParaKeys".into())
    }

    fn display_path(&self) -> String {
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

    fn reload(&mut self) {
        self.keys.clear();
        self.revealed.clear();
        self.backend.clear();
        self.env_name.clear();
        self.has_vault = false;
        self.has_unlock = false;
        self.status.clear();

        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };

        if let Some(b) = detect_backend(&root) {
            self.backend = b.as_str().to_string();
        }
        self.has_vault = default_vault_path(&root).is_file();
        self.has_unlock = has_unlock_key(&root);

        if !self.has_vault || !self.has_unlock {
            return;
        }

        let key = match load_unlock_key(&root) {
            Ok(k) => k,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        let vault = match load_vault(&root, &key) {
            Ok(v) => v,
            Err(e) => {
                self.status = e.to_string();
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
            self.keys.push((name.clone(), st));
            if self.reveal {
                if let Some(v) = vault.get(&name) {
                    self.revealed.push((name, v.to_string()));
                }
            }
        }
        self.env_name = load_config(&root)
            .map(|c| c.env_name)
            .unwrap_or_else(|_| "local".into());
    }

    fn init_vault(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        if default_vault_path(&root).is_file() {
            self.status = "Vault already exists.".into();
            return;
        }
        let key = VaultKey::generate();
        if let Err(e) = save_vault(&root, &VaultData::new(), &key) {
            self.status = e.to_string();
            return;
        }
        match store_unlock_key(&root, &key) {
            Ok(outcome) => {
                self.recovery = encode_recovery_code(&key);
                let mut msg = match outcome.backend {
                    WalletBackend::KeychainUserPresence => {
                        "Vault created. Keychain unlock with Touch ID when available.".into()
                    }
                    WalletBackend::Keychain => "Vault created. Unlock key in Keychain.".to_string(),
                    WalletBackend::File => "Vault created. Unlock key in file wallet.".to_string(),
                };
                for n in &outcome.notes {
                    msg.push('\n');
                    msg.push_str(n);
                }
                self.status = msg;
                self.reload();
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    fn import_env(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
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
                self.status = if s.is_empty() {
                    "Imported.".into()
                } else {
                    s
                };
                self.reload();
            }
            Ok(o) => self.status = String::from_utf8_lossy(&o.stderr).trim().to_string(),
            Err(e) => self.status = e.to_string(),
        }
    }

    fn run_cmd(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let cmd = self.run_cmd.trim();
        if cmd.is_empty() {
            self.status = "Enter a command.".into();
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
                self.status = if o.status.success() {
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
            Err(e) => self.status = e.to_string(),
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme(ctx);

        // Top bar — single strip, like a small utility window
        egui::TopBottomPanel::top("bar")
            .exact_height(52.0)
            .frame(
                Frame::new()
                    .fill(C::WINDOW)
                    .inner_margin(Margin::symmetric(16, 10))
                    .stroke(Stroke::new(1.0, C::HAIRLINE)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.title()).size(15.0).strong().color(C::LABEL));
                        let mut sub = self.display_path();
                        if !self.backend.is_empty() {
                            if !sub.is_empty() {
                                sub.push_str("  ·  ");
                            }
                            sub.push_str(&self.backend);
                        }
                        if !sub.is_empty() {
                            ui.label(RichText::new(sub).size(11.0).color(C::TERTIARY));
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if btn_secondary(ui, "Open…").clicked() {
                            if let Some(f) = rfd::FileDialog::new().pick_folder() {
                                self.path = f.display().to_string();
                                self.reload();
                            }
                        }
                        if self.has_vault && self.has_unlock {
                            if btn_secondary(ui, "Import").clicked() {
                                self.import_env();
                            }
                            ui.add_space(4.0);
                            let r = ui.checkbox(
                                &mut self.reveal,
                                RichText::new("Show Values").size(12.0).color(C::SECONDARY),
                            );
                            if r.changed() || self.reveal != self.prev_reveal {
                                self.prev_reveal = self.reveal;
                                self.reload();
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(C::WINDOW)
                    .inner_margin(Margin::symmetric(20, 16)),
            )
            .show(ctx, |ui| {
                // Recovery (only when just created)
                if !self.recovery.is_empty() {
                    Frame::new()
                        .fill(Color32::from_rgb(255, 250, 240))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(230, 200, 160)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Recovery code — store offline")
                                    .size(12.0)
                                    .strong()
                                    .color(C::ORANGE),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(&self.recovery)
                                    .size(13.0)
                                    .monospace()
                                    .color(C::LABEL),
                            );
                        });
                    ui.add_space(12.0);
                }

                if !self.status.is_empty() {
                    ui.label(RichText::new(&self.status).size(12.0).color(C::SECONDARY));
                    ui.add_space(8.0);
                }

                // ── States ──────────────────────────────────────────────
                if !self.has_vault {
                    // Centered empty — one action only
                    let h = ui.available_height();
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), h),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.add_space((h * 0.28).clamp(40.0, 120.0));
                            ui.label(
                                RichText::new("No Vault")
                                    .size(22.0)
                                    .color(C::LABEL)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(
                                    "Create a vault to keep this project’s environment\nsecrets out of .env files.",
                                )
                                .size(13.0)
                                .color(C::SECONDARY),
                            );
                            ui.add_space(20.0);
                            if btn_primary(ui, "Create Vault").clicked() {
                                self.init_vault();
                            }
                        },
                    );
                } else if !self.has_unlock {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(RichText::new("Locked").size(20.0).strong().color(C::LABEL));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Restore unlock from Terminal:")
                                .size(13.0)
                                .color(C::SECONDARY),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("parakeys init --recover '<code>'")
                                .size(12.0)
                                .monospace()
                                .color(C::TERTIARY),
                        );
                    });
                } else if self.keys.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(72.0);
                        ui.label(RichText::new("No Keys").size(20.0).strong().color(C::LABEL));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Import a .env to move secrets into the vault.")
                                .size(13.0)
                                .color(C::SECONDARY),
                        );
                        ui.add_space(18.0);
                        if btn_primary(ui, "Import .env").clicked() {
                            self.import_env();
                        }
                    });
                } else {
                    // Run (only with keys/vault)
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Run").size(12.0).color(C::TERTIARY));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.run_cmd)
                                .hint_text("Command")
                                .desired_width(ui.available_width() - 70.0)
                                .font(FontId::proportional(13.0)),
                        );
                        if btn_primary(ui, "Run").clicked() {
                            self.run_cmd();
                        }
                    });
                    ui.add_space(12.0);

                    // Inset grouped list (iOS Settings / Passwords list feel)
                    Frame::new()
                        .fill(C::SURFACE)
                        .stroke(Stroke::new(1.0, C::HAIRLINE))
                        .corner_radius(CornerRadius::same(8))
                        .show(ui, |ui| {
                            let n = self.keys.len();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, (name, st)) in self.keys.iter().enumerate() {
                                    let val = self
                                        .revealed
                                        .iter()
                                        .find(|(k, _)| k == name)
                                        .map(|(_, v)| v.clone());

                                    ui.horizontal(|ui| {
                                        ui.add_space(14.0);
                                        ui.vertical(|ui| {
                                            ui.add_space(11.0);
                                            ui.label(
                                                RichText::new(name)
                                                    .size(14.0)
                                                    .color(C::LABEL),
                                            );
                                            if self.reveal {
                                                if let Some(v) = val {
                                                    ui.label(
                                                        RichText::new(v)
                                                            .size(12.0)
                                                            .monospace()
                                                            .color(C::BLUE),
                                                    );
                                                } else {
                                                    chip(ui, *st);
                                                }
                                            } else {
                                                chip(ui, *st);
                                            }
                                            ui.add_space(11.0);
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(12.0);
                                                if !self.reveal {
                                                    // trailing chevron-ish quiet detail
                                                    ui.label(
                                                        RichText::new("›")
                                                            .size(18.0)
                                                            .color(C::TERTIARY),
                                                    );
                                                }
                                            },
                                        );
                                    });
                                    if i + 1 < n {
                                        let y = ui.cursor().top();
                                        let rect = ui.max_rect();
                                        ui.painter().hline(
                                            (rect.left() + 14.0)..=rect.right(),
                                            y,
                                            Stroke::new(1.0, C::ROW_SEP),
                                        );
                                    }
                                }
                            });
                        });

                    if !self.env_name.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("{}  ·  {} keys", self.env_name, self.keys.len()))
                                .size(11.0)
                                .color(C::TERTIARY),
                        );
                    }
                }
            });
    }
}
