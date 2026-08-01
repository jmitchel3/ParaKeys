//! ParaKeys desktop GUI (Passwords-like project env surface).

use std::path::PathBuf;
use std::process::Command;

use eframe::egui;
use parakeys::config::load_config;
use parakeys::envfile::load_env_file;
use parakeys::keywallet::{
    detect_backend, encode_recovery_code, has_unlock_key, load_unlock_key, project_root,
    store_unlock_key, WalletBackend,
};
use parakeys::status::{classify_key, status_label};
use parakeys::vault::{default_vault_path, load_vault, save_vault, VaultData, VaultKey};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_title("ParaKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "ParaKeys",
        options,
        Box::new(|_cc| Ok(Box::new(ParaKeysApp::default()))),
    )
}

#[derive(Default)]
struct ParaKeysApp {
    project_path: String,
    status: String,
    keys: Vec<(String, String)>, // name, status label
    reveal: bool,
    revealed: Vec<(String, String)>,
    recovery_shown: String,
    run_cmd: String,
    last_backend: String,
}

impl ParaKeysApp {
    fn root(&self) -> Result<PathBuf, String> {
        let p = self.project_path.trim();
        if p.is_empty() {
            return project_root(None).map_err(|e| e.to_string());
        }
        Ok(PathBuf::from(p))
    }

    fn refresh_keys(&mut self) {
        self.keys.clear();
        self.revealed.clear();
        self.last_backend.clear();
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
            self.status = format!("No vault at {} — use Init", vault_path.display());
            return;
        }
        if !has_unlock_key(&root) {
            self.status = "No unlock key (Init or recover with CLI).".into();
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
                self.status = format!("Decrypt failed: {e}");
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
        let mut names: std::collections::BTreeSet<String> =
            vault.keys.keys().cloned().collect();
        names.extend(env_map.keys().cloned());
        for name in names {
            let env_v = env_map.get(&name).map(String::as_str);
            let st = classify_key(&vault, &name, env_v);
            self.keys
                .push((name.clone(), status_label(st).to_string()));
            if self.reveal {
                if let Some(val) = vault.get(&name) {
                    self.revealed.push((name, val.to_string()));
                }
            }
        }
        let env_name = load_config(&root)
            .map(|c| c.env_name)
            .unwrap_or_else(|_| "default".into());
        self.status = format!(
            "Loaded {} key(s) · env={} · wallet={}",
            self.keys.len(),
            env_name,
            if self.last_backend.is_empty() {
                "?"
            } else {
                &self.last_backend
            }
        );
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
            self.status = "Vault already exists (use CLI --force to recreate).".into();
            return;
        }
        let key = VaultKey::generate();
        if let Err(e) = save_vault(&root, &VaultData::new(), &key) {
            self.status = format!("init vault failed: {e}");
            return;
        }
        match store_unlock_key(&root, &key) {
            Ok(backend) => {
                self.recovery_shown = encode_recovery_code(&key);
                self.status = match backend {
                    WalletBackend::Keychain => {
                        "Initialized vault; unlock key in Keychain. Save recovery code below."
                            .into()
                    }
                    WalletBackend::File => {
                        "Initialized vault; unlock key in file wallet. Save recovery code below."
                            .into()
                    }
                };
                self.refresh_keys();
            }
            Err(e) => self.status = format!("store unlock key failed: {e}"),
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
        let status = Command::new(std::env::current_exe().ok().and_then(|p| {
            // Prefer sibling `parakeys` CLI next to this GUI binary.
            let mut cli = p.clone();
            cli.set_file_name("parakeys");
            if cli.is_file() {
                Some(cli)
            } else {
                None
            }
        }).unwrap_or_else(|| PathBuf::from("parakeys")))
        .args(["import", "--path", &root.display().to_string(), env_path.to_str().unwrap_or(".env")])
        .output();
        match status {
            Ok(out) if out.status.success() => {
                self.status = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if self.status.is_empty() {
                    self.status = "Import complete.".into();
                }
                self.refresh_keys();
            }
            Ok(out) => {
                self.status = format!(
                    "import failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
            Err(e) => self.status = format!("import spawn failed: {e} (is parakeys CLI on PATH?)"),
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
            self.status = "Enter a command to run (e.g. printenv FOO).".into();
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
                    format!("run ok:\n{stdout}")
                } else {
                    format!("run failed:\n{stderr}\n{stdout}")
                };
            }
            Err(e) => self.status = format!("run spawn failed: {e}"),
        }
    }
}

impl eframe::App for ParaKeysApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ParaKeys");
                ui.label("Like Apple Passwords, but for dotenv");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Project:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.project_path)
                        .desired_width(420.0)
                        .hint_text("path (empty = current directory)"),
                );
                if ui.button("Browse…").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.project_path = folder.display().to_string();
                        self.refresh_keys();
                    }
                }
                if ui.button("Refresh").clicked() {
                    self.refresh_keys();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Init vault").clicked() {
                    self.do_init();
                }
                if ui.button("Import .env").clicked() {
                    self.do_import();
                }
                ui.checkbox(&mut self.reveal, "Reveal secrets");
                if self.reveal {
                    self.refresh_keys();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Run:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.run_cmd)
                        .desired_width(360.0)
                        .hint_text("command e.g. printenv DATABASE_URL"),
                );
                if ui.button("parakeys run").clicked() {
                    self.do_run();
                }
            });

            ui.separator();
            ui.label(&self.status);

            if !self.recovery_shown.is_empty() {
                ui.group(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(180, 80, 40),
                        "RECOVERY CODE (store offline; shown once):",
                    );
                    ui.monospace(&self.recovery_shown);
                });
            }

            ui.separator();
            ui.heading("Keys");
            if self.keys.is_empty() {
                ui.label("(no keys — Init / Import / Refresh)");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("keys")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("Name");
                            ui.strong(if self.reveal { "Value" } else { "Status" });
                            ui.end_row();
                            if self.reveal && !self.revealed.is_empty() {
                                for (n, v) in &self.revealed {
                                    ui.monospace(n);
                                    ui.monospace(v);
                                    ui.end_row();
                                }
                            } else {
                                for (n, st) in &self.keys {
                                    ui.monospace(n);
                                    ui.label(st);
                                    ui.end_row();
                                }
                            }
                        });
                });
            }

            ui.separator();
            ui.small(format!(
                "Wallet: {} · Status labels never dump secrets unless Reveal is on.",
                if self.last_backend.is_empty() {
                    "unknown"
                } else {
                    &self.last_backend
                }
            ));
        });
    }
}
