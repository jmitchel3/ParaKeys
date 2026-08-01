//! ParaKeys desktop GUI — Passwords-like density, less toolkit chrome.

use std::path::PathBuf;
use std::process::Command;

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Sense, Stroke, Vec2,
};
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, status_label, KeyStatus};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

// System-adjacent dark: closer to macOS Passwords / Settings night mode.
struct Palette;
impl Palette {
    const BG: Color32 = Color32::from_rgb(28, 28, 30);
    const BG_SIDE: Color32 = Color32::from_rgb(36, 36, 38);
    const BG_CARD: Color32 = Color32::from_rgb(44, 44, 46);
    const BG_INPUT: Color32 = Color32::from_rgb(54, 54, 56);
    const BG_HOVER: Color32 = Color32::from_rgb(58, 58, 60);
    const STROKE: Color32 = Color32::from_rgb(58, 58, 60);
    const DIVIDER: Color32 = Color32::from_rgb(56, 56, 58);
    const TEXT: Color32 = Color32::from_rgb(255, 255, 255);
    const TEXT_SEC: Color32 = Color32::from_rgb(152, 152, 157);
    const TEXT_TERT: Color32 = Color32::from_rgb(110, 110, 115);
    const BLUE: Color32 = Color32::from_rgb(10, 132, 255);
    const BLUE_FILL: Color32 = Color32::from_rgb(10, 132, 255);
    const GREEN: Color32 = Color32::from_rgb(48, 209, 88);
    const ORANGE: Color32 = Color32::from_rgb(255, 159, 10);
    const SELECT: Color32 = Color32::from_rgb(48, 48, 52);
}

fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(Palette::TEXT);
    v.panel_fill = Palette::BG;
    v.window_fill = Palette::BG;
    v.extreme_bg_color = Palette::BG_INPUT;
    v.faint_bg_color = Palette::BG_CARD;
    v.widgets.noninteractive.bg_fill = Palette::BG_CARD;
    v.widgets.inactive.bg_fill = Palette::BG_CARD;
    v.widgets.hovered.bg_fill = Palette::BG_HOVER;
    v.widgets.active.bg_fill = Palette::BLUE.gamma_multiply(0.35);
    v.widgets.inactive.bg_stroke = Stroke::new(0.5, Palette::STROKE);
    v.widgets.hovered.bg_stroke = Stroke::new(0.5, Palette::STROKE);
    v.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Palette::STROKE);
    v.selection.bg_fill = Palette::SELECT;
    v.window_corner_radius = CornerRadius::same(10);
    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}

fn blue_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::WHITE).size(13.0).strong())
            .fill(Palette::BLUE_FILL)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

fn gray_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Palette::TEXT).size(13.0))
            .fill(Palette::BG_CARD)
            .stroke(Stroke::new(0.5, Palette::STROKE))
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

fn link_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Palette::BLUE).size(13.0))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE),
    )
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(text)
            .size(11.0)
            .color(Palette::TEXT_TERT)
            .strong(),
    );
    ui.add_space(4.0);
}

fn status_chip(ui: &mut egui::Ui, status: KeyStatus) {
    let (label, fg, bg) = match status {
        KeyStatus::SetInVault => ("Set", Palette::GREEN, Color32::from_rgb(20, 40, 24)),
        KeyStatus::NotSet => ("Not set", Palette::TEXT_TERT, Color32::from_rgb(48, 48, 50)),
        KeyStatus::PlaintextOnDisk => ("On disk", Palette::ORANGE, Color32::from_rgb(48, 36, 12)),
    };
    Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(11.0).color(fg));
        });
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 600.0])
            .with_min_inner_size([700.0, 440.0])
            .with_title("ParaKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "ParaKeys",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            let mut app = ParaKeysApp::default();
            // Default to cwd so the app is useful immediately.
            if let Ok(cwd) = project_root(None) {
                app.project_path = cwd.display().to_string();
                app.refresh_keys();
            }
            Ok(Box::new(app))
        }),
    )
}

struct ParaKeysApp {
    project_path: String,
    status: String,
    keys: Vec<(String, String, KeyStatus)>,
    reveal: bool,
    prev_reveal: bool,
    revealed: Vec<(String, String)>,
    recovery_shown: String,
    run_cmd: String,
    last_backend: String,
    env_name: String,
    has_vault: bool,
    has_unlock: bool,
}

impl Default for ParaKeysApp {
    fn default() -> Self {
        Self {
            project_path: String::new(),
            status: String::new(),
            keys: Vec::new(),
            reveal: false,
            prev_reveal: false,
            revealed: Vec::new(),
            recovery_shown: String::new(),
            run_cmd: String::new(),
            last_backend: String::new(),
            env_name: String::new(),
            has_vault: false,
            has_unlock: false,
        }
    }
}

impl ParaKeysApp {
    fn root(&self) -> Result<PathBuf, String> {
        let p = self.project_path.trim();
        if p.is_empty() {
            return project_root(None).map_err(|e| e.to_string());
        }
        Ok(PathBuf::from(p))
    }

    fn project_title(&self) -> String {
        self.root()
            .ok()
            .and_then(|r| r.file_name().map(|s| s.to_string_lossy().into_owned()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "No Project".into())
    }

    fn short_path(&self) -> String {
        let p = self.project_path.trim();
        if p.is_empty() {
            return "Choose a folder".into();
        }
        // Collapse home for display
        if let Ok(home) = std::env::var("HOME") {
            if let Some(rest) = p.strip_prefix(&home) {
                return format!("~{rest}");
            }
        }
        p.to_string()
    }

    fn refresh_keys(&mut self) {
        self.keys.clear();
        self.revealed.clear();
        self.last_backend.clear();
        self.env_name.clear();
        self.has_vault = false;
        self.has_unlock = false;

        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };

        if let Some(b) = detect_backend(&root) {
            self.last_backend = b.as_str().to_string();
        }
        self.has_vault = default_vault_path(&root).is_file();
        self.has_unlock = has_unlock_key(&root);

        if !self.has_vault {
            self.status = String::new();
            return;
        }
        if !self.has_unlock {
            self.status = "This vault needs an unlock key. Recover from the CLI if needed.".into();
            return;
        }

        let key = match load_unlock_key(&root) {
            Ok(k) => k,
            Err(e) => {
                self.status = format!("Unlock failed: {e}");
                return;
            }
        };
        let vault = match load_vault(&root, &key) {
            Ok(v) => v,
            Err(e) => {
                self.status = format!("Could not open vault: {e}");
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

        let mut names: std::collections::BTreeSet<String> = vault.keys.keys().cloned().collect();
        names.extend(env_map.keys().cloned());
        for name in names {
            let env_v = env_map.get(&name).map(String::as_str);
            let st = classify_key(&vault, &name, env_v);
            self.keys
                .push((name.clone(), status_label(st).to_string(), st));
            if self.reveal {
                if let Some(val) = vault.get(&name) {
                    self.revealed.push((name, val.to_string()));
                }
            }
        }

        self.env_name = load_config(&root)
            .map(|c| c.env_name)
            .unwrap_or_else(|_| "local".into());
        self.status = String::new();
    }

    fn do_init(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        if default_vault_path(&root).is_file() {
            self.status = "A vault already exists for this project.".into();
            return;
        }
        let key = VaultKey::generate();
        if let Err(e) = save_vault(&root, &VaultData::new(), &key) {
            self.status = format!("Could not create vault: {e}");
            return;
        }
        match store_unlock_key(&root, &key) {
            Ok(outcome) => {
                self.recovery_shown = encode_recovery_code(&key);
                let mut msg = match outcome.backend {
                    WalletBackend::KeychainUserPresence => {
                        "Vault created. Unlock uses Keychain with Touch ID when available."
                            .to_string()
                    }
                    WalletBackend::Keychain => {
                        "Vault created. Unlock key stored in Keychain.".to_string()
                    }
                    WalletBackend::File => {
                        "Vault created. Unlock key stored in the local file wallet.".to_string()
                    }
                };
                for n in &outcome.notes {
                    msg.push('\n');
                    msg.push_str(n);
                }
                self.status = msg;
                self.refresh_keys();
            }
            Err(e) => self.status = format!("Could not store unlock key: {e}"),
        }
    }

    fn do_import(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let env_path = root.join(".env");
        let cli = sibling_cli();
        let out = Command::new(cli)
            .args([
                "import",
                "--path",
                &root.display().to_string(),
                env_path.to_str().unwrap_or(".env"),
            ])
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                self.status = if s.is_empty() {
                    "Imported. Values are no longer on disk.".into()
                } else {
                    s
                };
                self.refresh_keys();
            }
            Ok(o) => {
                self.status = format!("{}", String::from_utf8_lossy(&o.stderr).trim());
            }
            Err(e) => self.status = format!("Could not run parakeys: {e}"),
        }
    }

    fn do_run(&mut self) {
        let root = match self.root() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let cmd = self.run_cmd.trim();
        if cmd.is_empty() {
            self.status = "Enter a command to run.".into();
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
        match Command::new(sibling_cli()).args(&args).output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                self.status = if o.status.success() {
                    if stdout.trim().is_empty() {
                        "Done.".into()
                    } else {
                        stdout.trim().to_string()
                    }
                } else if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else {
                    stdout.trim().to_string()
                };
            }
            Err(e) => self.status = format!("Could not run: {e}"),
        }
    }
}

fn sibling_cli() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            let mut c = p;
            c.set_file_name("parakeys");
            c.is_file().then_some(c)
        })
        .unwrap_or_else(|| PathBuf::from("parakeys"))
}

impl eframe::App for ParaKeysApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // ── Sidebar (Passwords-style list chrome) ─────────────────────────
        egui::SidePanel::left("side")
            .exact_width(220.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(Palette::BG_SIDE)
                    .inner_margin(Margin::symmetric(12, 12)),
            )
            .show(ctx, |ui| {
                // Title row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("ParaKeys").size(15.0).strong().color(Palette::TEXT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if link_btn(ui, "Open…").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.project_path = folder.display().to_string();
                                self.refresh_keys();
                            }
                        }
                    });
                });
                ui.label(
                    RichText::new(self.short_path())
                        .size(11.0)
                        .color(Palette::TEXT_TERT),
                );

                ui.add_space(14.0);
                section_label(ui, "Library");

                // Selection row: All Keys
                let selected = self.has_vault;
                let fill = if selected {
                    Palette::SELECT
                } else {
                    Color32::TRANSPARENT
                };
                Frame::new()
                    .fill(fill)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("●").size(8.0).color(Palette::BLUE));
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("All Keys")
                                        .size(13.0)
                                        .color(Palette::TEXT)
                                        .strong(),
                                );
                                let sub = if !self.has_vault {
                                    "No vault"
                                } else if self.keys.is_empty() {
                                    "Empty"
                                } else {
                                    // count only
                                    ""
                                };
                                if sub.is_empty() {
                                    ui.label(
                                        RichText::new(format!(
                                            "{} item{}",
                                            self.keys.len(),
                                            if self.keys.len() == 1 { "" } else { "s" }
                                        ))
                                        .size(11.0)
                                        .color(Palette::TEXT_SEC),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new(sub).size(11.0).color(Palette::TEXT_SEC),
                                    );
                                }
                            });
                        });
                    });

                ui.add_space(6.0);
                Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("○").size(8.0).color(Palette::TEXT_TERT));
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(self.project_title())
                                        .size(13.0)
                                        .color(Palette::TEXT_SEC),
                                );
                                let wallet = if self.last_backend.is_empty() {
                                    if self.has_vault {
                                        "locked"
                                    } else {
                                        "—"
                                    }
                                } else {
                                    &self.last_backend
                                };
                                ui.label(
                                    RichText::new(format!("Wallet · {wallet}"))
                                        .size(11.0)
                                        .color(Palette::TEXT_TERT),
                                );
                            });
                        });
                    });

                ui.add_space(16.0);
                section_label(ui, "Privacy");
                let reveal_resp = ui.checkbox(
                    &mut self.reveal,
                    RichText::new("Show values").size(13.0).color(Palette::TEXT_SEC),
                );
                if reveal_resp.changed() || self.reveal != self.prev_reveal {
                    self.prev_reveal = self.reveal;
                    self.refresh_keys();
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(4.0);
                    if !self.has_vault {
                        if blue_btn(ui, "Create Vault").clicked() {
                            self.do_init();
                        }
                        ui.add_space(6.0);
                    } else {
                        if gray_btn(ui, "Import .env").clicked() {
                            self.do_import();
                        }
                        ui.add_space(6.0);
                        if gray_btn(ui, "Refresh").clicked() {
                            self.refresh_keys();
                        }
                    }
                });
            });

        // ── Main content ──────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(Palette::BG)
                    .inner_margin(Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                // Title: only when vault exists
                if self.has_vault {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("All Keys")
                                .size(22.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        if !self.env_name.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(self.env_name.as_str())
                                    .size(13.0)
                                    .color(Palette::TEXT_SEC),
                            );
                        }
                    });
                    ui.add_space(12.0);

                    // Run only when vault is ready
                    if self.has_unlock {
                        Frame::new()
                            .fill(Palette::BG_CARD)
                            .stroke(Stroke::new(0.5, Palette::STROKE))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Run with secrets")
                                            .size(12.0)
                                            .color(Palette::TEXT_TERT),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.run_cmd)
                                            .hint_text("e.g. npm start")
                                            .desired_width(ui.available_width() - 72.0)
                                            .font(FontId::proportional(13.0)),
                                    );
                                    if blue_btn(ui, "Run").clicked() {
                                        self.do_run();
                                    }
                                });
                            });
                        ui.add_space(14.0);
                    }
                }

                // Status / recovery
                if !self.recovery_shown.is_empty() {
                    Frame::new()
                        .fill(Color32::from_rgb(44, 34, 20))
                        .stroke(Stroke::new(0.5, Palette::ORANGE.gamma_multiply(0.5)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Save this recovery code")
                                    .size(12.0)
                                    .strong()
                                    .color(Palette::ORANGE),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(&self.recovery_shown)
                                    .size(13.0)
                                    .monospace()
                                    .color(Palette::TEXT),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("Store it offline. It will not be shown again.")
                                    .size(11.0)
                                    .color(Palette::TEXT_SEC),
                            );
                        });
                    ui.add_space(12.0);
                }

                if !self.status.is_empty() && self.has_vault {
                    ui.label(RichText::new(&self.status).size(12.0).color(Palette::TEXT_SEC));
                    ui.add_space(8.0);
                }

                // Empty / content
                if !self.has_vault {
                    // Full empty: hero CTA only (no Run bar, no orphan count)
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.22);
                        // Soft circle
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
                        ui.painter()
                            .circle_filled(r.center(), 28.0, Palette::BG_CARD);
                        ui.painter().circle_stroke(
                            r.center(),
                            28.0,
                            Stroke::new(1.0, Palette::STROKE),
                        );
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            "key",
                            FontId::proportional(14.0),
                            Palette::TEXT_SEC,
                        );
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("No Vault")
                                .size(20.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Create a vault for this project to store\nenvironment secrets securely.")
                                .size(13.0)
                                .color(Palette::TEXT_SEC),
                        );
                        ui.add_space(18.0);
                        if blue_btn(ui, "  Create Vault  ").clicked() {
                            self.do_init();
                        }
                        ui.add_space(8.0);
                        if link_btn(ui, "Choose a different folder").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.project_path = folder.display().to_string();
                                self.refresh_keys();
                            }
                        }
                    });
                } else if !self.has_unlock {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(
                            RichText::new("Vault Locked")
                                .size(18.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Use recovery from the CLI:\nparakeys init --recover '<code>'")
                                .size(13.0)
                                .color(Palette::TEXT_SEC)
                                .monospace(),
                        );
                    });
                } else if self.keys.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.label(
                            RichText::new("No Keys")
                                .size(18.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Import a .env file to move secrets into the vault.")
                                .size(13.0)
                                .color(Palette::TEXT_SEC),
                        );
                        ui.add_space(16.0);
                        if blue_btn(ui, "  Import .env  ").clicked() {
                            self.do_import();
                        }
                    });
                } else {
                    // List
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            Frame::new()
                                .fill(Palette::BG_CARD)
                                .stroke(Stroke::new(0.5, Palette::STROKE))
                                .corner_radius(CornerRadius::same(10))
                                .show(ui, |ui| {
                                    let n = self.keys.len();
                                    for (i, (name, _st, ks)) in self.keys.iter().enumerate() {
                                        let val = self
                                            .revealed
                                            .iter()
                                            .find(|(k, _)| k == name)
                                            .map(|(_, v)| v.clone());

                                        ui.horizontal(|ui| {
                                            ui.add_space(12.0);
                                            ui.vertical(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(
                                                    RichText::new(name)
                                                        .size(14.0)
                                                        .color(Palette::TEXT),
                                                );
                                                if self.reveal {
                                                    if let Some(v) = val {
                                                        ui.label(
                                                            RichText::new(v)
                                                                .size(12.0)
                                                                .monospace()
                                                                .color(Palette::BLUE),
                                                        );
                                                    } else {
                                                        status_chip(ui, *ks);
                                                    }
                                                } else {
                                                    status_chip(ui, *ks);
                                                }
                                                ui.add_space(10.0);
                                            });
                                        });
                                        if i + 1 < n {
                                            ui.painter().hline(
                                                ui.max_rect().x_range(),
                                                ui.cursor().top(),
                                                Stroke::new(0.5, Palette::DIVIDER),
                                            );
                                        }
                                    }
                                });
                        });
                }
            });
    }
}
