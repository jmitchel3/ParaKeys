//! ParaKeys desktop GUI — refined Passwords-like surface with a quiet Grok signal.

use std::path::PathBuf;
use std::process::Command;

use eframe::egui::{self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Sense, Stroke, Vec2};
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, status_label, KeyStatus};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

// ─── Design tokens: “Apple vault, night sky, one signal” ───────────────────
// Not generic cream, not acid cyberpunk. Cool graphite + soft violet haze +
// a single electric teal used sparingly (the “signal”).

struct Palette;
impl Palette {
    const BG: Color32 = Color32::from_rgb(18, 18, 22);
    const BG_ELEVATED: Color32 = Color32::from_rgb(28, 28, 34);
    const BG_SIDE: Color32 = Color32::from_rgb(22, 22, 28);
    const BG_ROW: Color32 = Color32::from_rgb(32, 32, 40);
    const BG_ROW_HOVER: Color32 = Color32::from_rgb(40, 40, 50);
    const STROKE: Color32 = Color32::from_rgb(55, 55, 68);
    const TEXT: Color32 = Color32::from_rgb(242, 242, 247);
    const TEXT_DIM: Color32 = Color32::from_rgb(142, 142, 158);
    const TEXT_MUTED: Color32 = Color32::from_rgb(99, 99, 112);
    const SIGNAL: Color32 = Color32::from_rgb(90, 200, 250); // cool system blue-cyan
    const SIGNAL_SOFT: Color32 = Color32::from_rgb(40, 80, 110);
    const SUCCESS: Color32 = Color32::from_rgb(52, 199, 89);
    const WARN: Color32 = Color32::from_rgb(255, 159, 10);
    const RECOVERY_BG: Color32 = Color32::from_rgb(48, 32, 20);
    const RECOVERY_STROKE: Color32 = Color32::from_rgb(180, 100, 40);
}

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Palette::TEXT);
    visuals.widgets.noninteractive.bg_fill = Palette::BG_ELEVATED;
    visuals.widgets.inactive.bg_fill = Palette::BG_ROW;
    visuals.widgets.hovered.bg_fill = Palette::BG_ROW_HOVER;
    visuals.widgets.active.bg_fill = Palette::SIGNAL_SOFT;
    visuals.widgets.open.bg_fill = Palette::BG_ROW;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Palette::TEXT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Palette::TEXT);
    visuals.selection.bg_fill = Palette::SIGNAL_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, Palette::SIGNAL);
    visuals.panel_fill = Palette::BG;
    visuals.window_fill = Palette::BG;
    visuals.extreme_bg_color = Palette::BG_SIDE;
    visuals.faint_bg_color = Palette::BG_ELEVATED;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Palette::STROKE);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Palette::STROKE);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Palette::SIGNAL.gamma_multiply(0.5));
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.button_frame = true;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.indent = 18.0;
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}

fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::from_rgb(10, 20, 30)).strong())
            .fill(Palette::SIGNAL)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Palette::TEXT))
            .fill(Palette::BG_ROW)
            .stroke(Stroke::new(1.0, Palette::STROKE))
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 32.0)),
    )
}

fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Palette::TEXT_DIM).size(13.0))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(6)),
    )
}

fn status_pill(ui: &mut egui::Ui, status: &str) {
    let (fg, bg) = if status.contains("set in parakeys") && !status.contains("not set") {
        (Palette::SUCCESS, Color32::from_rgb(20, 48, 28))
    } else if status.contains("not set") {
        (Palette::TEXT_DIM, Color32::from_rgb(40, 40, 48))
    } else if status.contains("plaintext") {
        (Palette::WARN, Color32::from_rgb(48, 36, 16))
    } else {
        (Palette::TEXT_DIM, Palette::BG_ROW)
    };
    Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(status).size(11.0).color(fg));
        });
}

fn monogram(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), 18.0, Palette::SIGNAL_SOFT);
    painter.circle_stroke(rect.center(), 18.0, Stroke::new(1.0, Palette::SIGNAL.gamma_multiply(0.6)));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "pk",
        FontId::proportional(14.0),
        Palette::SIGNAL,
    );
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 640.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("ParaKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "ParaKeys",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(ParaKeysApp::default()))
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
    key_count: usize,
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
            key_count: 0,
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

    fn project_label(&self) -> String {
        if let Ok(root) = self.root() {
            root.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Project")
                .to_string()
        } else {
            "Project".into()
        }
    }

    fn refresh_keys(&mut self) {
        self.keys.clear();
        self.revealed.clear();
        self.last_backend.clear();
        self.env_name.clear();
        self.key_count = 0;
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
        let vault_path = default_vault_path(&root);
        if !vault_path.is_file() {
            self.status = "No vault yet. Create one to begin.".into();
            return;
        }
        if !has_unlock_key(&root) {
            self.status = "No unlock key. Init a vault or recover with the CLI.".into();
            return;
        }
        let key = match load_unlock_key(&root) {
            Ok(k) => k,
            Err(e) => {
                self.status = format!("Could not unlock: {e}");
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
        let env_path = root.join(".env");
        if env_path.is_file() {
            if let Ok(env) = load_env_file(&env_path) {
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
        self.key_count = self.keys.len();
        self.env_name = load_config(&root)
            .map(|c| c.env_name)
            .unwrap_or_else(|_| "default".into());
        self.status = if self.key_count == 0 {
            "Vault is ready. Import a .env or add keys from the CLI.".into()
        } else {
            format!(
                "{} key{} · {} · {}",
                self.key_count,
                if self.key_count == 1 { "" } else { "s" },
                self.env_name,
                if self.last_backend.is_empty() {
                    "locked"
                } else {
                    &self.last_backend
                }
            )
        };
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
            self.status = "This project already has a vault.".into();
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
                        "Vault created. Unlock key is in the Keychain.".to_string()
                    }
                    WalletBackend::File => {
                        "Vault created. Unlock key is in the local file wallet.".to_string()
                    }
                };
                for n in &outcome.notes {
                    msg.push_str("\n");
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
        let status = Command::new(
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    let mut cli = p;
                    cli.set_file_name("parakeys");
                    if cli.is_file() {
                        Some(cli)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| PathBuf::from("parakeys")),
        )
        .args([
            "import",
            "--path",
            &root.display().to_string(),
            env_path.to_str().unwrap_or(".env"),
        ])
        .output();
        match status {
            Ok(out) if out.status.success() => {
                self.status = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if self.status.is_empty() {
                    self.status = "Imported secrets. Values stay off disk.".into();
                }
                self.refresh_keys();
            }
            Ok(out) => {
                self.status = format!(
                    "Import failed: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            Err(e) => {
                self.status = format!("Could not run parakeys CLI: {e}");
            }
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
            self.status = "Enter a command to run with secrets injected.".into();
            return;
        }
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut args = vec![
            "run".to_string(),
            "--path".to_string(),
            root.display().to_string(),
            "--".to_string(),
        ];
        args.extend(parts.iter().map(|s| s.to_string()));
        let cli = std::env::current_exe()
            .ok()
            .and_then(|p| {
                let mut c = p;
                c.set_file_name("parakeys");
                if c.is_file() {
                    Some(c)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| PathBuf::from("parakeys"));
        match Command::new(cli).args(&args).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                self.status = if out.status.success() {
                    if stdout.trim().is_empty() {
                        "Command finished.".into()
                    } else {
                        format!("{}", stdout.trim())
                    }
                } else {
                    format!("{}", stderr.trim().or_empty(&stdout))
                };
            }
            Err(e) => self.status = format!("Could not run command: {e}"),
        }
    }
}

trait OrEmpty {
    fn or_empty<'a>(&'a self, other: &'a str) -> &'a str;
}
impl OrEmpty for str {
    fn or_empty<'a>(&'a self, other: &'a str) -> &'a str {
        if self.is_empty() {
            other
        } else {
            self
        }
    }
}

impl eframe::App for ParaKeysApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // ── Left rail ─────────────────────────────────────────────────────
        egui::SidePanel::left("rail")
            .exact_width(240.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(Palette::BG_SIDE)
                    .stroke(Stroke::new(1.0, Palette::STROKE))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    monogram(ui);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("ParaKeys")
                                .size(17.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        ui.label(
                            RichText::new("project secrets")
                                .size(11.0)
                                .color(Palette::TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(28.0);
                ui.label(
                    RichText::new("PROJECT")
                        .size(10.0)
                        .color(Palette::TEXT_MUTED)
                        .strong(),
                );
                ui.add_space(6.0);

                Frame::new()
                    .fill(Palette::BG_ELEVATED)
                    .stroke(Stroke::new(1.0, Palette::STROKE))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(self.project_label())
                                .size(14.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        if !self.env_name.is_empty() {
                            ui.label(
                                RichText::new(format!("env · {}", self.env_name))
                                    .size(11.0)
                                    .color(Palette::SIGNAL),
                            );
                        }
                        let wallet = if self.last_backend.is_empty() {
                            "not unlocked"
                        } else {
                            &self.last_backend
                        };
                        ui.label(
                            RichText::new(format!("wallet · {wallet}"))
                                .size(11.0)
                                .color(Palette::TEXT_MUTED),
                        );
                    });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if secondary_button(ui, "Browse").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.project_path = folder.display().to_string();
                            self.refresh_keys();
                        }
                    }
                    if ghost_button(ui, "Refresh").clicked() {
                        self.refresh_keys();
                    }
                });

                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.project_path)
                        .hint_text("Project path")
                        .desired_width(f32::INFINITY)
                        .text_color(Palette::TEXT_DIM)
                        .frame(true),
                );

                ui.add_space(24.0);
                ui.label(
                    RichText::new("ACTIONS")
                        .size(10.0)
                        .color(Palette::TEXT_MUTED)
                        .strong(),
                );
                ui.add_space(8.0);

                if primary_button(ui, "  Create vault  ").clicked() {
                    self.do_init();
                }
                ui.add_space(6.0);
                if secondary_button(ui, "  Import .env  ").clicked() {
                    self.do_import();
                }

                ui.add_space(16.0);
                let reveal_response = ui.checkbox(
                    &mut self.reveal,
                    RichText::new("Reveal secrets").size(13.0).color(Palette::TEXT_DIM),
                );
                if reveal_response.changed() || self.reveal != self.prev_reveal {
                    self.prev_reveal = self.reveal;
                    self.refresh_keys();
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Secrets stay off disk.\nStatus never dumps values.")
                            .size(11.0)
                            .color(Palette::TEXT_MUTED),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Like Apple Passwords,\nrefined for dotenv.")
                            .size(11.0)
                            .color(Palette::TEXT_MUTED)
                            .italics(),
                    );
                });
            });

        // ── Main ──────────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(Palette::BG)
                    .inner_margin(Margin::symmetric(28, 24)),
            )
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Keys")
                                .size(28.0)
                                .strong()
                                .color(Palette::TEXT),
                        );
                        if !self.status.is_empty() {
                            ui.label(
                                RichText::new(&self.status)
                                    .size(13.0)
                                    .color(Palette::TEXT_DIM),
                            );
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{}", self.key_count))
                                .size(32.0)
                                .color(Palette::SIGNAL)
                                .strong(),
                        );
                    });
                });

                ui.add_space(16.0);

                // Run bar
                Frame::new()
                    .fill(Palette::BG_ELEVATED)
                    .stroke(Stroke::new(1.0, Palette::STROKE))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Run")
                                    .size(12.0)
                                    .color(Palette::TEXT_MUTED)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.run_cmd)
                                    .hint_text("command · secrets inject into the child process")
                                    .desired_width(ui.available_width() - 100.0)
                                    .font(FontId::monospace(13.0)),
                            );
                            if primary_button(ui, "Run").clicked() {
                                self.do_run();
                            }
                        });
                    });

                // Recovery banner
                if !self.recovery_shown.is_empty() {
                    ui.add_space(14.0);
                    Frame::new()
                        .fill(Palette::RECOVERY_BG)
                        .stroke(Stroke::new(1.0, Palette::RECOVERY_STROKE))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Recovery code · store offline · shown once")
                                    .size(11.0)
                                    .color(Palette::WARN)
                                    .strong(),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(&self.recovery_shown)
                                    .size(14.0)
                                    .monospace()
                                    .color(Palette::TEXT),
                            );
                        });
                }

                ui.add_space(20.0);

                // Key list
                if self.keys.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(48.0);
                        monogram(ui);
                        ui.add_space(16.0);
                        ui.label(
                            RichText::new("Nothing here yet")
                                .size(18.0)
                                .color(Palette::TEXT)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Create a vault, then import a .env.\nValues never stay on disk as plaintext.")
                                .size(13.0)
                                .color(Palette::TEXT_MUTED)
                                .italics(),
                        );
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (i, (name, st, _ks)) in self.keys.iter().enumerate() {
                                let value_opt = self
                                    .revealed
                                    .iter()
                                    .find(|(n, _)| n == name)
                                    .map(|(_, v)| v.as_str());

                                Frame::new()
                                    .fill(if i % 2 == 0 {
                                        Palette::BG_ROW
                                    } else {
                                        Palette::BG_ELEVATED
                                    })
                                    .stroke(Stroke::new(1.0, Palette::STROKE.gamma_multiply(0.6)))
                                    .corner_radius(CornerRadius::same(10))
                                    .inner_margin(Margin::symmetric(14, 11))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Accent bar
                                            let (bar, _) = ui.allocate_exact_size(
                                                Vec2::new(3.0, 28.0),
                                                Sense::hover(),
                                            );
                                            ui.painter().rect_filled(
                                                bar,
                                                CornerRadius::same(2),
                                                Palette::SIGNAL.gamma_multiply(0.85),
                                            );
                                            ui.add_space(10.0);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(name)
                                                        .size(14.0)
                                                        .monospace()
                                                        .color(Palette::TEXT)
                                                        .strong(),
                                                );
                                                if self.reveal {
                                                    if let Some(v) = value_opt {
                                                        ui.label(
                                                            RichText::new(v)
                                                                .size(12.0)
                                                                .monospace()
                                                                .color(Palette::SIGNAL),
                                                        );
                                                    } else {
                                                        status_pill(ui, st);
                                                    }
                                                } else {
                                                    status_pill(ui, st);
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(6.0);
                            }
                        });
                }
            });
    }
}
