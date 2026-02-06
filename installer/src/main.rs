use gtk4::prelude::*;
use gtk4::{glib, Application, Stack};
use libadwaita as adw;
use adw::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::io;

const APP_ID: &str = "com.obision.appinstall.Installer";

struct AppState {
    lis_file: Option<PathBuf>,
    metadata: Option<liblis::Metadata>,
    current_screen: usize,
    install_for_all_users: bool,  // false = actual user, true = for all users
}

impl AppState {
    fn new() -> Self {
        Self {
            lis_file: None,
            metadata: None,
            current_screen: 0,
            install_for_all_users: false,  // Default to actual user
        }
    }
}

fn main() -> glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Handle command line arguments
    app.connect_open(move |app, files, _| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                // Start installer with the provided .lis file
                start_installer_with_file(app, path);
            }
        }
    });

    app.run()
}

fn build_ui(app: &Application) {
    let app_state = Rc::new(RefCell::new(AppState::new()));
    
    // Check if a file was passed as argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let lis_path = PathBuf::from(&args[1]);
        if lis_path.exists() && lis_path.extension().and_then(|s| s.to_str()) == Some("lis") {
            app_state.borrow_mut().lis_file = Some(lis_path.clone());
            show_wizard(app, app_state);
            return;
        }
    }
    
    // No file provided, show file selection screen
    show_file_selector(app, app_state);
}

fn start_installer_with_file(app: &Application, path: PathBuf) {
    let app_state = Rc::new(RefCell::new(AppState::new()));
    app_state.borrow_mut().lis_file = Some(path);
    show_wizard(app, app_state);
}

/// Helper functions to build FHS-compliant installation paths
mod install_paths {
    use std::path::PathBuf;
    
    /// Get the bin directory for the given prefix
    pub fn bin_dir(prefix: &str) -> PathBuf {
        PathBuf::from(prefix).join("bin")
    }
    
    /// Get the share directory for the given prefix
    pub fn share_dir(prefix: &str) -> PathBuf {
        PathBuf::from(prefix).join("share")
    }
    
    /// Get the applications directory (for .desktop files)
    pub fn applications_dir(prefix: &str) -> PathBuf {
        share_dir(prefix).join("applications")
    }
    
    /// Get the app-specific data directory
    pub fn app_data_dir(prefix: &str, app_id: &str) -> PathBuf {
        share_dir(prefix).join(app_id)
    }
    
    /// Get the icons directory
    pub fn icons_dir(prefix: &str) -> PathBuf {
        share_dir(prefix).join("icons").join("hicolor")
    }
    
    /// Get the metainfo directory
    pub fn metainfo_dir(prefix: &str) -> PathBuf {
        share_dir(prefix).join("metainfo")
    }
    
    /// Expand ~ to home directory
    pub fn expand_home(path: &str) -> String {
        if path.starts_with("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return home.to_string_lossy().to_string() + &path[1..];
            }
        }
        path.to_string()
    }
}

/// Installation manifest for uninstallation
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct InstallationManifest {
    /// Application ID
    app_id: String,
    /// Application name
    app_name: String,
    /// Version
    version: String,
    /// Installation prefix used
    prefix: String,
    /// Timestamp of installation
    installed_at: String,
    /// List of all installed files (absolute paths)
    installed_files: Vec<String>,
    /// List of all created directories (absolute paths)
    created_directories: Vec<String>,
}

impl InstallationManifest {
    fn new(metadata: &liblis::Metadata, prefix: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            app_id: metadata.package.app_id.clone(),
            app_name: metadata.package.application_name.clone(),
            version: metadata.package.version.clone(),
            prefix,
            installed_at: format!("{}", now),
            installed_files: Vec::new(),
            created_directories: Vec::new(),
        }
    }
    
    /// Get the registry directory for installation manifests
    fn registry_dir(for_all_users: bool) -> PathBuf {
        if for_all_users {
            // System-wide registry
            PathBuf::from("/var/lib/obision-installer/manifests")
        } else {
            // User registry
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".local/share/obision-installer/manifests")
        }
    }
    
    /// Get the manifest file path for this app
    fn manifest_path(&self, for_all_users: bool) -> PathBuf {
        Self::registry_dir(for_all_users).join(format!("{}.json", self.app_id))
    }
    
    /// Save the manifest to disk
    fn save(&self, for_all_users: bool) -> Result<(), String> {
        let manifest_path = self.manifest_path(for_all_users);
        
        // Create registry directory if it doesn't exist
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create manifest directory: {}", e))?;
        }
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        
        // Write to file
        fs::write(&manifest_path, json)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;
        
        Ok(())
    }
}

/// Perform the actual installation
fn perform_installation(
    metadata: &liblis::Metadata,
    lis_file: &PathBuf,
    for_all_users: bool,
    progress_callback: impl Fn(f64, &str),
) -> Result<(), String> {
    // Determine installation prefix
    let prefix = if for_all_users {
        install_paths::expand_home(&metadata.installation.prefix_system)
    } else {
        install_paths::expand_home(&metadata.installation.prefix_user)
    };
    
    progress_callback(0.1, &format!("Installing to {}...", prefix));
    
    // TODO: Extract .lis file (tar.gz) to temporary directory
    // For now, we'll create the directory structure
    
    // Create necessary directories
    let bin_dir = install_paths::bin_dir(&prefix);
    let share_dir = install_paths::share_dir(&prefix);
    let app_data_dir = install_paths::app_data_dir(&prefix, &metadata.package.app_id);
    let applications_dir = install_paths::applications_dir(&prefix);
    let icons_dir = install_paths::icons_dir(&prefix);
    
    progress_callback(0.2, "Creating directories...");
    
    // Create directories
    fs::create_dir_all(&bin_dir).map_err(|e| format!("Failed to create bin directory: {}", e))?;
    fs::create_dir_all(&share_dir).map_err(|e| format!("Failed to create share directory: {}", e))?;
    fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app data directory: {}", e))?;
    fs::create_dir_all(&applications_dir).map_err(|e| format!("Failed to create applications directory: {}", e))?;
    fs::create_dir_all(&icons_dir).map_err(|e| format!("Failed to create icons directory: {}", e))?;
    
    progress_callback(0.5, "Extracting files...");
    
    // TODO: Extract files from .lis archive
    // This would involve:
    // 1. Opening the .lis file (tar.gz)
    // 2. Extracting to temp directory
    // 3. Copying files to their destinations based on metadata.files
    
    progress_callback(0.8, "Setting permissions...");
    
    // TODO: Set executable permissions on binaries
    
    progress_callback(1.0, "Installation complete!");
    
    Ok(())
}

fn show_file_selector(app: &Application, app_state: Rc<RefCell<AppState>>) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(600)
        .default_height(400)
        .title("Obision Installer")
        .build();

    let header = adw::HeaderBar::builder().build();
    
    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    
    content.append(&header);
    
    // Status page with file selection button
    let status_page = adw::StatusPage::builder()
        .icon_name("system-software-install-symbolic")
        .title("Obision Package Installer")
        .description("Select a .lis package to install")
        .vexpand(true)
        .build();
    
    let select_button = gtk4::Button::builder()
        .label("Select .lis Package")
        .halign(gtk4::Align::Center)
        .build();
    
    select_button.add_css_class("suggested-action");
    select_button.add_css_class("pill");
    
    status_page.set_child(Some(&select_button));
    content.append(&status_page);
    
    window.set_content(Some(&content));
    
    // Handle file selection
    let window_clone = window.clone();
    let app_clone = app.clone();
    let app_state_clone = app_state.clone();
    
    select_button.connect_clicked(move |_| {
        let file_chooser = gtk4::FileChooserDialog::new(
            Some("Select .lis Package"),
            Some(&window_clone),
            gtk4::FileChooserAction::Open,
            &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)],
        );
        
        let filter = gtk4::FileFilter::new();
        filter.add_pattern("*.lis");
        filter.set_name(Some("Obision Packages"));
        file_chooser.add_filter(&filter);
        
        let window_clone2 = window_clone.clone();
        let app_clone2 = app_clone.clone();
        let app_state_clone2 = app_state_clone.clone();
        
        file_chooser.connect_response(move |dialog, response| {
            if response == gtk4::ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        app_state_clone2.borrow_mut().lis_file = Some(path);
                        window_clone2.close();
                        show_wizard(&app_clone2, app_state_clone2.clone());
                    }
                }
            }
            dialog.close();
        });
        
        file_chooser.show();
    });
    
    window.present();
}

fn show_wizard(app: &Application, app_state: Rc<RefCell<AppState>>) {
    // Extract and load metadata from .lis file
    let _lis_path = match &app_state.borrow().lis_file {
        Some(path) => path.clone(),
        None => return,
    };
    
    // TODO: Extract metadata.toml from .lis and parse it
    // For now, create a dummy metadata for testing
    let metadata = create_dummy_metadata();
    app_state.borrow_mut().metadata = Some(metadata.clone());
    
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(700)
        .default_height(550)
        .title(&format!("Installing {}", metadata.package.application_name))
        .build();

    let main_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();

    let header = adw::HeaderBar::builder().build();
    main_box.append(&header);

    // Create stack for wizard screens
    let stack = Stack::builder()
        .vexpand(true)
        .transition_type(gtk4::StackTransitionType::SlideLeftRight)
        .build();

    // Get enabled screens from metadata (clone them to avoid lifetime issues)
    let mut screens: Vec<liblis::metadata::InstallerScreen> = metadata.installer_screens.iter()
        .filter(|s| s.enabled)
        .cloned()
        .collect();
    screens.sort_by_key(|s| s.order);
    
    // Always insert progress screen before finish (if not already present)
    if !screens.iter().any(|s| s.id == "progress") {
        // Find the finish screen index
        if let Some(finish_idx) = screens.iter().position(|s| s.id == "finish") {
            screens.insert(finish_idx, liblis::metadata::InstallerScreen {
                id: "progress".to_string(),
                enabled: true,
                order: screens[finish_idx].order,
                custom_content: None,
            });
        }
    }

    // Create screens
    for screen in &screens {
        let screen_widget = create_screen(&screen.id, &metadata, app_state.clone());
        stack.add_named(&screen_widget, Some(&screen.id));
    }

    main_box.append(&stack);

    // Navigation buttons
    let button_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .halign(gtk4::Align::End)
        .build();

    let back_button = gtk4::Button::builder()
        .label("Back")
        .build();

    let next_button = gtk4::Button::builder()
        .label("Next")
        .build();
    next_button.add_css_class("suggested-action");

    button_box.append(&back_button);
    button_box.append(&next_button);
    main_box.append(&button_box);

    window.set_content(Some(&main_box));

    // Show first screen
    if let Some(first_screen) = screens.first() {
        stack.set_visible_child_name(&first_screen.id);
        back_button.set_sensitive(false);
    }

    // Navigation logic
    let stack_clone = stack.clone();
    let app_state_clone = app_state.clone();
    let back_button_clone = back_button.clone();
    let next_button_clone = next_button.clone();
    let screens_clone = screens.clone();
    let window_clone = window.clone();

    next_button.connect_clicked(move |_| {
        let mut state = app_state_clone.borrow_mut();
        if state.current_screen < screens_clone.len() - 1 {
            state.current_screen += 1;
            stack_clone.set_visible_child_name(&screens_clone[state.current_screen].id);
            back_button_clone.set_sensitive(true);
            
            // Change button text on last screen
            if state.current_screen == screens_clone.len() - 1 {
                next_button_clone.set_label("Finish");
            }
        } else {
            // Finish installation
            window_clone.close();
        }
    });

    let stack_clone2 = stack.clone();
    let app_state_clone2 = app_state.clone();
    let next_button_clone2 = next_button.clone();
    let screens_clone2 = screens.clone();

    back_button.connect_clicked(move |btn| {
        let mut state = app_state_clone2.borrow_mut();
        if state.current_screen > 0 {
            state.current_screen -= 1;
            stack_clone2.set_visible_child_name(&screens_clone2[state.current_screen].id);
            next_button_clone2.set_label("Next");
            
            if state.current_screen == 0 {
                btn.set_sensitive(false);
            }
        }
    });

    window.present();
}

fn create_screen(screen_id: &str, metadata: &liblis::Metadata, app_state: Rc<RefCell<AppState>>) -> gtk4::Widget {
    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .vexpand(true)
        .build();

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(24)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(24)
        .margin_end(24)
        .valign(gtk4::Align::Center)
        .build();

    match screen_id {
        "welcome" => {
            let status_page = adw::StatusPage::builder()
                .icon_name("emblem-ok-symbolic")
                .title(&format!("Welcome to {} Setup", metadata.package.application_name))
                .description(&format!(
                    "This wizard will guide you through the installation of {}.\n\nVersion: {}\nAuthor: {}",
                    metadata.package.application_name,
                    metadata.package.version,
                    metadata.package.author
                ))
                .build();
            content_box.append(&status_page);
        }
        "license" => {
            let status_page = adw::StatusPage::builder()
                .icon_name("text-x-generic-symbolic")
                .title("License Agreement")
                .description("Please read the following license agreement carefully.")
                .build();
            content_box.append(&status_page);
            
            // TODO: Add license text from custom_content
            let license_text = gtk4::TextView::builder()
                .editable(false)
                .wrap_mode(gtk4::WrapMode::Word)
                .build();
            license_text.buffer().set_text("License text would go here...");
            
            let scrolled = gtk4::ScrolledWindow::builder()
                .child(&license_text)
                .min_content_height(200)
                .build();
            content_box.append(&scrolled);
        }
        "install_location" => {
            let status_page = adw::StatusPage::builder()
                .icon_name("folder-symbolic")
                .title("Installation User")
                .description("Choose where to install the application.")
                .build();
            content_box.append(&status_page);
            
            // Create radio buttons - Actual user first (default)
            let user_check = gtk4::CheckButton::builder()
                .label("Actual user")
                .active(true)
                .build();
            
            let system_check = gtk4::CheckButton::builder()
                .label("For all users")
                .group(&user_check)
                .build();
            
            // Connect radio buttons to update app_state
            {
                let app_state = app_state.clone();
                user_check.connect_toggled(move |btn| {
                    if btn.is_active() {
                        app_state.borrow_mut().install_for_all_users = false;
                    }
                });
            }
            
            {
                let app_state = app_state.clone();
                system_check.connect_toggled(move |btn| {
                    if btn.is_active() {
                        app_state.borrow_mut().install_for_all_users = true;
                    }
                });
            }
            
            let check_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(12)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .build();
            
            check_box.append(&user_check);
            check_box.append(&system_check);
            
            let list_box = gtk4::ListBox::builder()
                .selection_mode(gtk4::SelectionMode::None)
                .build();
            list_box.add_css_class("boxed-list");
            list_box.append(&check_box);
            
            content_box.append(&list_box);
        }
        "progress" => {
            let status_page = adw::StatusPage::builder()
                .icon_name("emblem-synchronizing-symbolic")
                .title("Installing")
                .description(&format!("Installing {}...", metadata.package.application_name))
                .build();
            content_box.append(&status_page);
            
            let progress = gtk4::ProgressBar::builder()
                .fraction(0.5)
                .build();
            content_box.append(&progress);
        }
        "finish" => {
            let status_page = adw::StatusPage::builder()
                .icon_name("emblem-default-symbolic")
                .title("Installation Complete")
                .description(&format!(
                    "{} has been successfully installed on your system.",
                    metadata.package.application_name
                ))
                .build();
            content_box.append(&status_page);
        }
        _ => {
            let label = gtk4::Label::new(Some(&format!("Unknown screen: {}", screen_id)));
            content_box.append(&label);
        }
    }

    clamp.set_child(Some(&content_box));
    clamp.upcast()
}

// Temporary function to create dummy metadata for testing
fn create_dummy_metadata() -> liblis::Metadata {
    liblis::Metadata {
        package: liblis::metadata::PackageInfo {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            app_id: "com.example.testapp".to_string(),
            description: "A test application".to_string(),
            author: "Test Author".to_string(),
            application_name: "Test Application".to_string(),
            package_name: "testapp.lis".to_string(),
            compression_level: 9,
        },
        installation: liblis::metadata::InstallationInfo {
            prefix_system: "/usr/local".to_string(),
            prefix_user: "~/.local".to_string(),
        },
        desktop: liblis::metadata::DesktopInfo {
            name: "Test Application".to_string(),
            exec: "testapp".to_string(),
            icon: "".to_string(),
            categories: vec!["Utility".to_string()],
        },
        dependencies: liblis::metadata::DependenciesInfo {
            bundled: vec![],
        },
        files: vec![],
        installer_screens: vec![
            liblis::metadata::InstallerScreen {
                id: "welcome".to_string(),
                enabled: true,
                order: 1,
                custom_content: None,
            },
            liblis::metadata::InstallerScreen {
                id: "license".to_string(),
                enabled: true,
                order: 2,
                custom_content: None,
            },
            liblis::metadata::InstallerScreen {
                id: "install_location".to_string(),
                enabled: true,
                order: 3,
                custom_content: None,
            },
            liblis::metadata::InstallerScreen {
                id: "finish".to_string(),
                enabled: true,
                order: 4,
                custom_content: None,
            },
        ],
    }
}
