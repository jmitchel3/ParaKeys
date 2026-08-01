//! Application state and three-pane layout built on the design system.

use std::path::PathBuf;
use std::process::Command;

use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Margin, RichText, Sense, Stroke, Vec2};

use super::ds::{
    self, card, card_warning, category_tile, detail_field, empty_state, hairline, list_row,
    panel_list, panel_main, panel_side, panel_toolbar, primary_button, project_card, search_field,
    section_label, status_accent, status_chip, text_body_secondary, text_caption, text_hero,
    text_title, toolbar_pill, Color, Layout, Radius, Space, Type,
};
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, KeyStatus};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    All,
    Set,
    Missing,
    OnDisk,
}

impl Category {
    fn meta(self) -> (&'static str, &'static str, Color32) {
        match self {
            Self::All => ("All", "key", Color::ACCENT),
            Self::Set => ("Set", "✓", Color::SUCCESS),
            Self::Missing => ("Missing", "?", Color::WARNING),
            Self::OnDisk => ("On Disk", "!", Color::DANGER),
        }
    }
}

struct KeyRow {
    name: String,
    status: KeyStatus,
    value: Option<String>,
}

pub struct App {
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
    pub fn new() -> Self {
        let mut app = Self::default();
        if let Ok(cwd) = project_root(None) {
            app.path = cwd.display().to_string();
            app.reload();
        }
        app
    }

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
                    Category::All => true,
                    Category::Set => k.status == KeyStatus::SetInVault,
                    Category::Missing => k.status == KeyStatus::NotSet,
                    Category::OnDisk => k.status == KeyStatus::PlaintextOnDisk,
                };
                let search_ok = q.is_empty() || k.name.to_ascii_lowercase().contains(&q);
                cat_ok && search_ok
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn reload(&mut self) {
        let prev = self
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
        self.selected = prev.and_then(|n| self.keys.iter().position(|k| k.name == n));
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

fn status_short(s: KeyStatus) -> &'static str {
    match s {
        KeyStatus::SetInVault => "Set in vault",
        KeyStatus::NotSet => "Not set",
        KeyStatus::PlaintextOnDisk => "Still in .env",
    }
}

fn status_kind(s: KeyStatus) -> u8 {
    match s {
        KeyStatus::SetInVault => 0,
        KeyStatus::NotSet => 1,
        KeyStatus::PlaintextOnDisk => 2,
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ds::apply_theme(ctx);
        let (n_all, n_set, n_miss, n_disk) = self.counts();

        // Toolbar
        egui::TopBottomPanel::top("toolbar")
            .exact_height(Layout::TOOLBAR_H)
            .frame(panel_toolbar())
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        let title = self.category.meta().0;
                        ui.label(text_title(title));
                        let count = match self.category {
                            Category::All => n_all,
                            Category::Set => n_set,
                            Category::Missing => n_miss,
                            Category::OnDisk => n_disk,
                        };
                        ui.label(text_caption(format!(
                            "{count} item{}",
                            if count == 1 { "" } else { "s" }
                        )));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        search_field(ui, &mut self.search, 168.0);
                        ui.add_space(Space::XS);
                        if toolbar_pill(ui, "Edit").clicked() { /* reserved */ }
                        if toolbar_pill(ui, "+").clicked() {
                            self.import_env();
                        }
                        if toolbar_pill(ui, "Open…").clicked() {
                            if let Some(f) = rfd::FileDialog::new().pick_folder() {
                                self.path = f.display().to_string();
                                self.reload();
                            }
                        }
                    });
                });
            });

        // Categories
        egui::SidePanel::left("cats")
            .exact_width(Layout::SIDEBAR_W)
            .resizable(false)
            .frame(panel_side())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (t, g, c) = Category::All.meta();
                    if category_tile(ui, t, n_all, g, c, self.category == Category::All).clicked()
                    {
                        self.category = Category::All;
                    }
                    ui.add_space(Space::SM);
                    let (t, g, c) = Category::Set.meta();
                    if category_tile(ui, t, n_set, g, c, self.category == Category::Set).clicked() {
                        self.category = Category::Set;
                    }
                });
                ui.add_space(Space::SM);
                ui.horizontal(|ui| {
                    let (t, g, c) = Category::Missing.meta();
                    if category_tile(ui, t, n_miss, g, c, self.category == Category::Missing)
                        .clicked()
                    {
                        self.category = Category::Missing;
                    }
                    ui.add_space(Space::SM);
                    let (t, g, c) = Category::OnDisk.meta();
                    if category_tile(ui, t, n_disk, g, c, self.category == Category::OnDisk)
                        .clicked()
                    {
                        self.category = Category::OnDisk;
                    }
                });

                section_label(ui, "Project");
                let wallet = if self.backend.is_empty() {
                    if self.has_vault {
                        "locked"
                    } else {
                        "no vault"
                    }
                } else {
                    self.backend.as_str()
                };
                let _ = project_card(ui, &self.project_name(), wallet, Color::WARNING);
                ui.add_space(Space::XS);
                ui.label(text_caption(self.short_path()));

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    let r = ui.checkbox(
                        &mut self.reveal,
                        RichText::new("Show values")
                            .size(Type::BODY)
                            .color(Color::TEXT_SECONDARY),
                    );
                    if r.changed() || self.reveal != self.prev_reveal {
                        self.prev_reveal = self.reveal;
                        self.reload();
                    }
                });
            });

        // List
        egui::SidePanel::left("list")
            .exact_width(Layout::LIST_W)
            .resizable(false)
            .frame(panel_list())
            .show(ctx, |ui| {
                if !self.has_vault || !self.has_unlock {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Space::XXXL * 3.0);
                        ui.label(text_body_secondary(if !self.has_vault {
                            "No items"
                        } else {
                            "Locked"
                        }));
                    });
                    return;
                }
                let indices = self.filtered_indices();
                if indices.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Space::XXXL * 3.0);
                        ui.label(text_body_secondary("No items"));
                    });
                    return;
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(Space::SM);
                        for &ki in &indices {
                            let row = &self.keys[ki];
                            let selected = self.selected == Some(ki);
                            let (accent, glyph) = status_accent(status_kind(row.status));
                            if list_row(
                                ui,
                                &row.name,
                                status_short(row.status),
                                glyph,
                                accent,
                                selected,
                            )
                            .clicked()
                            {
                                self.selected = Some(ki);
                            }
                            ui.add_space(2.0);
                        }
                        ui.add_space(Space::SM);
                    });
            });

        // Detail
        egui::CentralPanel::default()
            .frame(panel_main())
            .show(ctx, |ui| {
                if !self.recovery.is_empty() {
                    card_warning().show(ui, |ui| {
                        ui.label(
                            RichText::new("Recovery code")
                                .size(Type::CAPTION)
                                .strong()
                                .color(Color::WARNING),
                        );
                        ui.add_space(Space::XS);
                        ui.label(
                            RichText::new(&self.recovery)
                                .size(Type::CALL_OUT)
                                .monospace()
                                .color(Color::TEXT),
                        );
                        ui.label(text_caption("Store offline. Shown once."));
                    });
                    ui.add_space(Space::LG);
                }

                if !self.status_msg.is_empty() {
                    ui.label(text_body_secondary(&self.status_msg));
                    ui.add_space(Space::SM);
                }

                if !self.has_vault {
                    if empty_state(
                        ui,
                        "No Vault",
                        "Create a vault for this project to store\nenvironment secrets securely.",
                        Some("Create Vault"),
                    ) {
                        self.init_vault();
                    }
                    return;
                }
                if !self.has_unlock {
                    if empty_state(
                        ui,
                        "Locked",
                        "Restore unlock from Terminal:\nparakeys init --recover '<code>'",
                        None,
                    ) {
                    }
                    return;
                }
                if self.keys.is_empty() {
                    if empty_state(
                        ui,
                        "No Keys",
                        "Import a .env to move secrets into the vault.",
                        Some("Import .env"),
                    ) {
                        self.import_env();
                    }
                    return;
                }

                let idx = self.selected.or(self.filtered_indices().first().copied());
                let Some(ki) = idx else { return };
                let Some(row) = self.keys.get(ki) else { return };

                // Hero
                ui.vertical_centered(|ui| {
                    let (accent, _) = status_accent(status_kind(row.status));
                    let (r, _) =
                        ui.allocate_exact_size(Vec2::splat(Layout::ICON_LG + 8.0), Sense::hover());
                    ui.painter().rect(
                        r,
                        CornerRadius::same(Radius::XXL),
                        accent,
                        Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        r.center(),
                        Align2::CENTER_CENTER,
                        "key",
                        FontId::proportional(Type::CALL_OUT),
                        Color::TEXT_ON_ACCENT,
                    );
                    ui.add_space(Space::MD);
                    ui.label(text_hero(&row.name));
                    ui.add_space(Space::XS);
                    status_chip(ui, status_short(row.status), accent);
                });

                ui.add_space(Space::XL);

                card()
                    .inner_margin(Margin::symmetric(Space::LG as i8, Space::XS as i8))
                    .show(ui, |ui| {
                        detail_field(ui, "Name", &row.name, false);
                        hairline(ui);
                        let val = if self.reveal {
                            row.value.clone().unwrap_or_else(|| "not set".into())
                        } else if row.status == KeyStatus::SetInVault {
                            "••••••••••••".into()
                        } else {
                            "not set".into()
                        };
                        detail_field(ui, "Value", &val, self.reveal);
                        hairline(ui);
                        detail_field(ui, "Status", status_short(row.status), false);
                        hairline(ui);
                        detail_field(
                            ui,
                            "Environment",
                            if self.env_name.is_empty() {
                                "default"
                            } else {
                                &self.env_name
                            },
                            false,
                        );
                        hairline(ui);
                        detail_field(
                            ui,
                            "Wallet",
                            if self.backend.is_empty() {
                                "none"
                            } else {
                                &self.backend
                            },
                            false,
                        );
                        hairline(ui);
                        detail_field(ui, "Project", &self.project_name(), false);
                    });

                ui.add_space(Space::LG);
                card().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(text_caption("Run with secrets"));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.run_cmd)
                                .hint_text("npm start")
                                .desired_width(ui.available_width() - 72.0)
                                .font(FontId::proportional(Type::BODY)),
                        );
                        if primary_button(ui, "Run").clicked() {
                            self.run_cmd();
                        }
                    });
                });
            });
    }
}

