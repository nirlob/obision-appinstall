use gtk4::prelude::*;
use gtk4::{glib, gio, Application, FileDialog, AlertDialog};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::fs::File;
use libadwaita as adw;
use adw::prelude::PreferencesGroupExt;
use adw::prelude::ActionRowExt;

mod project;
use project::{Project, ProjectFile};

struct AppState {
    project: Project,
    current_path: Option<PathBuf>,
    is_modified: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            project: Project::new(),
            current_path: None,
            is_modified: false,
        }
    }
}

const APP_ID: &str = "com.obision.appinstall.Builder";

/// Parse a .desktop file and extract Name and Comment
fn parse_desktop_file(path: &PathBuf) -> Result<(String, String), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read .desktop file: {}", e))?;
    
    let mut name = String::new();
    let mut comment = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") {
            name = line.strip_prefix("Name=").unwrap_or("").to_string();
        } else if line.starts_with("Comment=") {
            comment = line.strip_prefix("Comment=").unwrap_or("").to_string();
        }
    }
    
    if name.is_empty() {
        return Err("No Name field found in .desktop file".to_string());
    }
    
    Ok((name, comment))
}

fn main() -> glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.connect_startup(|app| {
        setup_actions(app);
    });

    app.run()
}

fn setup_actions(app: &Application) {
    let actions = ["new-project", "open-project", "save-project", "save-as-project", "about"];
    for action_name in actions {
        let action = gio::SimpleAction::new(action_name, None);
        app.add_action(&action);
    }
    
    // About handler can be global
    if let Some(action) = app.lookup_action("about") {
        if let Some(simple_action) = action.downcast_ref::<gio::SimpleAction>() {
            simple_action.connect_activate(|_, _| {
                println!("About clicked");
            });
        }
    }
}

fn build_ui(app: &Application) {
    let window_builder = gtk4::Builder::from_string(include_str!("../data/ui/window.ui"));
    let window: adw::ApplicationWindow = window_builder
        .object("window")
        .expect("Could not get window from UI file");
    
    window.set_application(Some(app));

    let app_state = Rc::new(RefCell::new(AppState::new()));
    setup_window_interaction(&window_builder, app_state);


    let root_stack: gtk4::Stack = window_builder.object("root_stack").expect("Could not get root_stack");
    show_welcome_screen(&window, &root_stack);
    
    window.present();
}

fn setup_window_interaction(builder: &gtk4::Builder, app_state: Rc<RefCell<AppState>>) {
    let window = builder.object::<adw::ApplicationWindow>("window").expect("Could not get window");
    let root_stack: gtk4::Stack = builder.object("root_stack").expect("Could not get root_stack");
    let content_header: adw::HeaderBar = builder.object("content_header").expect("Could not get content_header");
    let sidebar_list: gtk4::ListBox = builder.object("sidebar_list").expect("Could not get sidebar_list");
    let content_stack: gtk4::Stack = builder.object("content_stack").expect("Could not get content_stack");
    
    let project_name_entry: adw::EntryRow = builder.object("project_name_entry").expect("Could not get project_name_entry");
    let application_name_entry: adw::EntryRow = builder.object("application_name_entry").expect("Could not get application_name_entry");
    let project_version_entry: adw::EntryRow = builder.object("project_version_entry").expect("Could not get project_version_entry");
    let project_author_entry: adw::EntryRow = builder.object("project_author_entry").expect("Could not get project_author_entry");
    let project_description_entry: adw::EntryRow = builder.object("project_description_entry").expect("Could not get project_description_entry");
    let package_name_entry: adw::EntryRow = builder.object("package_name_entry").expect("Could not get package_name_entry");
    
    let files_group: adw::PreferencesGroup = builder.object("files_list_group").expect("Could not get files_list_group");
    let add_file_row: adw::ActionRow = builder.object("add_file_row").expect("Could not get add_file_row");
    
    let output_dir_row: adw::ActionRow = builder.object("output_dir_row").expect("Could not get output_dir_row");
    let output_dir_button: gtk4::Button = builder.object("output_dir_button").expect("output_dir_button");
    let build_package_button: gtk4::Button = builder.object("build_package_button").expect("build_package_button");
    let build_log_view: gtk4::TextView = builder.object("build_log_view").expect("build_log_view");
    
    // Installer screen switches
    let screen_welcome: adw::SwitchRow = builder.object("screen_welcome").expect("Could not get screen_welcome");
    let screen_license: adw::SwitchRow = builder.object("screen_license").expect("Could not get screen_license");
    let screen_install_location: adw::SwitchRow = builder.object("screen_install_location").expect("Could not get screen_install_location");
    let screen_finish: adw::SwitchRow = builder.object("screen_finish").expect("Could not get screen_finish");

    // == Helpers ==

    // Update Title
    let update_title = {
        let content_header = content_header.clone();
        let app_state = app_state.clone();
        move || {
            let state = app_state.borrow();
            let mut title = if let Some(path) = &state.current_path {
                path.to_string_lossy().into_owned()
            } else {
                "untitled.lisproj".to_string()
            };
            
            if state.is_modified {
                title.push('*');
            }
            
            content_header.set_title_widget(Some(&adw::WindowTitle::new(&title, "")));
        }
    };

    // Update Save Action State
    let update_save_action = {
        let window = window.clone();
        let app_state = app_state.clone();
        move || {
            if let Some(app) = window.application() {
                if let Some(action) = app.lookup_action("save-project") {
                    if let Some(simple) = action.downcast_ref::<gio::SimpleAction>() {
                        let state = app_state.borrow();
                        simple.set_enabled(state.is_modified);
                    }
                }
            }
        }
    };

    // Mark Modified
    let mark_modified = {
        let app_state = app_state.clone();
        let update_title = update_title.clone();
        let update_save_action = update_save_action.clone();
        move || {
            let mut state = app_state.borrow_mut();
            if !state.is_modified {
                state.is_modified = true;
                drop(state);
                update_title();
                update_save_action();
            }
        }
    };

    // Shared refresher container
    type Refresher = Rc<RefCell<Option<Rc<dyn Fn()>>>>;
    let refresher: Refresher = Rc::new(RefCell::new(None));

    // Helper wrapper to call refresh
    let call_refresh = {
        let refresher = refresher.clone();
        move || {
            if let Some(func) = refresher.borrow().as_ref() {
                func();
            }
        }
    };

    // Tracked rows for cleanup
    let active_file_rows: Rc<RefCell<Vec<gtk4::Widget>>> = Rc::new(RefCell::new(Vec::new()));

    // Define refresh logic
    let perform_refresh = {
        let files_group = files_group.clone();

        let app_state = app_state.clone();
        let mark_modified = mark_modified.clone();
        let window = window.clone();
        let refresher_weak = Rc::downgrade(&refresher);
        let active_file_rows = active_file_rows.clone();
        
        move || {
            // Remove previously added rows
            let mut tracked = active_file_rows.borrow_mut();
            for row in tracked.drain(..) {
                files_group.remove(&row);
            }
            
            let files = {
                 app_state.borrow().project.files.clone()
            };
            
            for (idx, file) in files.iter().enumerate() {
                let row = adw::ActionRow::builder()
                    .title(file.source.to_string_lossy().as_ref())
                    .subtitle(&file.destination)
                    .build();
                
                 let delete_btn = gtk4::Button::builder()
                    .icon_name("user-trash-symbolic")
                    .valign(gtk4::Align::Center)
                    .css_classes(["flat"])
                    .build();
                    
                 let app_state = app_state.clone();
                 let mark_modified = mark_modified.clone();
                 let window = window.clone();
                 let refresher_weak = refresher_weak.clone();

                 delete_btn.connect_clicked(move |_| {
                     let alert = AlertDialog::builder()
                        .modal(true)
                        .message("Remove File")
                        .detail("Are you sure you want to remove this file?")
                        .buttons(["Cancel", "Remove"])
                        .default_button(0)
                        .cancel_button(0)
                        .build();

                    let app_state = app_state.clone();
                    let mark_modified = mark_modified.clone();
                    let refresher_weak = refresher_weak.clone();
                    
                     alert.choose(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                        if let Ok(res) = result {
                            if res == 1 { 
                                app_state.borrow_mut().project.files.remove(idx);
                                mark_modified();
                                if let Some(refresher) = refresher_weak.upgrade() {
                                    if let Some(func) = refresher.borrow().as_ref() {
                                        func();
                                    }
                                }
                            }
                        }
                     });
                 });
                 
                 row.add_suffix(&delete_btn);
                 files_group.add(&row);
                 tracked.push(row.upcast());
            }
        }
    };
    
    *refresher.borrow_mut() = Some(Rc::new(perform_refresh));

    // Update UI from State
    let update_ui = {
        let app_state = app_state.clone();
        let call_refresh = call_refresh.clone();
        let name_entry = project_name_entry.clone();
        let app_name_entry = application_name_entry.clone(); // New
        let ver_entry = project_version_entry.clone();
        let auth_entry = project_author_entry.clone();
        let desc_entry = project_description_entry.clone(); // New
        let pkg_entry = package_name_entry.clone();
        let output_row = output_dir_row.clone();
        
        move || {
            let (name, app_name, ver, auth, desc, pkg, out_dir) = {
                let state = app_state.borrow();
                (
                    state.project.metadata.name.clone(),
                    state.project.metadata.application_name.clone(),
                    state.project.metadata.version.clone(),
                    state.project.metadata.author.clone(),
                    state.project.metadata.description.clone(),
                    state.project.package_name.clone(),
                    state.project.metadata.output_directory.clone(),
                )
            };
            
            name_entry.set_text(&name);
            app_name_entry.set_text(&app_name);
            ver_entry.set_text(&ver);
            auth_entry.set_text(&auth);
            desc_entry.set_text(&desc);
            pkg_entry.set_text(&pkg);
            output_row.set_subtitle(&out_dir.to_string_lossy());
            
            call_refresh();
        }
    };
    
    // Connect "Add File"
    add_file_row.connect_activated({
        let window = window.clone();
        let app_state = app_state.clone();
        let mark_modified = mark_modified.clone();
        let call_refresh = call_refresh.clone();
        move |_| {
            let file_dialog = FileDialog::builder().title("Add File").modal(true).build();
            let app_state = app_state.clone();
            let mark_modified = mark_modified.clone();
            let call_refresh = call_refresh.clone();
            
            file_dialog.open(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let mut state = app_state.borrow_mut();
                        state.project.files.push(ProjectFile {
                            source: path.clone(),
                            destination: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            permissions: None,
                        });
                        drop(state);
                        mark_modified();
                        call_refresh();
                    }
                }
            });
        }
    });

    // Connect Entry changes
    let entries = [
        (&project_name_entry, 0), 
        (&project_version_entry, 1), 
        (&project_author_entry, 2), 
        (&package_name_entry, 3),
        (&application_name_entry, 4), // New
        (&project_description_entry, 5) // New
    ];
    
    for (entry, id) in entries {
        let app_state = app_state.clone();
        let mark_modified = mark_modified.clone();
        entry.connect_notify_local(Some("text"), move |entry, _| {
            let text = entry.text().to_string();
            let mut state = app_state.borrow_mut();
            let changed = match id {
                0 => if state.project.metadata.name != text { state.project.metadata.name = text; true } else { false },
                1 => if state.project.metadata.version != text { state.project.metadata.version = text; true } else { false },
                2 => if state.project.metadata.author != text { state.project.metadata.author = text; true } else { false },
                3 => if state.project.package_name != text { state.project.package_name = text; true } else { false },
                4 => if state.project.metadata.application_name != text { state.project.metadata.application_name = text; true } else { false },
                5 => if state.project.metadata.description != text { state.project.metadata.description = text; true } else { false },
                _ => false,
            };
            drop(state);
            if changed {
                mark_modified();
            }
        });
    }

    // Connect Output Dir Button
    output_dir_button.connect_clicked({
        let window = window.clone();
        let app_state = app_state.clone();
        let mark_modified = mark_modified.clone();
        let update_ui = update_ui.clone();
        move |_| {
            let file_dialog = FileDialog::builder()
                .title("Select Output Directory")
                .modal(true)
                .build();
                
            let app_state = app_state.clone();
            let mark_modified = mark_modified.clone();
            let update_ui = update_ui.clone();
            
            file_dialog.select_folder(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                 if let Ok(file) = result {
                     if let Some(path) = file.path() {
                         let mut state = app_state.borrow_mut();
                         state.project.metadata.output_directory = path;
                         drop(state);
                         mark_modified();
                         update_ui();
                     }
                 }
            });
        }
    });

    // Connect Build Package Button


    // == Menu Actions Implementation ==
    
    // SAVE Logic
    let perform_save = {
        let app_state = app_state.clone();
        let update_title = update_title.clone();
        let update_save_action = update_save_action.clone();
        move |path: PathBuf| {
            let mut state = app_state.borrow_mut();
            if let Err(e) = state.project.save_to_file(&path) {
                 eprintln!("Save error: {}", e);
            } else {
                 println!("Saved to {:?}", path);
                 state.current_path = Some(path);
                 state.is_modified = false;
                 drop(state);
                 update_title();
                 update_save_action();
            }
        }
    };

    // Helper to start the save process (Save or Save As)
    let request_save = {
        let app_state = app_state.clone();
        let perform_save = perform_save.clone();
        let window = window.clone();
        move |on_success: Rc<dyn Fn()>| {
             let state = app_state.borrow();
             if let Some(path) = &state.current_path {
                 let path = path.clone();
                 drop(state);
                 let perform_save = perform_save.clone();
                 perform_save(path); // Synchronous mostly
                 on_success();
             } else {
                 drop(state);
                 // Save As
                 let file_dialog = FileDialog::builder().title("Save As").initial_name("myproject.lisproj").modal(true).build();
                 let filter = gtk4::FileFilter::new();
                 filter.add_pattern("*.lisproj");
                 filter.set_name(Some("Obision Project Files"));
                 let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                 filters.append(&filter);
                 file_dialog.set_filters(Some(&filters));
                 
                 let perform_save = perform_save.clone();
                 let on_success = on_success.clone();
                 
                 file_dialog.save(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                     if let Ok(file) = result {
                         if let Some(path) = file.path() {
                             perform_save(path);
                             on_success();
                         }
                     }
                 });
             }
        }
    };
    
    let check_unsaved = {
        let app_state = app_state.clone();
        let window = window.clone();
        let request_save = request_save.clone();
        let update_save_action = update_save_action.clone();
        
        move |on_proceed: Rc<dyn Fn()>| {
            let state = app_state.borrow();
            if state.is_modified {
                let alert = AlertDialog::builder()
                    .modal(true)
                    .message("Unsaved Changes")
                    .detail("You have unsaved changes. Do you want to save them before moving on?")
                    .buttons(["Cancel", "Discard", "Save"])
                    .default_button(2)
                    .cancel_button(0)
                    .build();
                
                let request_save = request_save.clone();
                let on_proceed_clone = on_proceed.clone();
                let app_state = app_state.clone();
                let update_save_action = update_save_action.clone();

                alert.choose(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                    if let Ok(res) = result {
                        if res == 1 { // Discard
                             // Mark as clean to proceed
                             let mut state = app_state.borrow_mut();
                             state.is_modified = false;
                             drop(state);
                             update_save_action();
                             on_proceed();
                        } else if res == 2 { // Save
                             request_save(on_proceed_clone);
                        }
                        // Cancel (0) -> do nothing
                    }
                });
            } else {
                drop(state);
                on_proceed();
            }
        }
    };
    
    // Connect Build Package Button
    // Connect Build Package Button
    build_package_button.connect_clicked({
        let app_state = app_state.clone();
        let window = window.clone();
        let check_unsaved = check_unsaved.clone();
        let build_log_view = build_log_view.clone();
        
        move |_| {
            let app_state = app_state.clone();
            let window = window.clone();
            let build_log_view = build_log_view.clone();
            
            // Build Logic Closure
            let run_build = move || {
                // Validate required fields
                let state = app_state.borrow();
                let validation_errors = {
                    let mut errors = Vec::new();
                    
                    if state.project.metadata.name.trim().is_empty() {
                        errors.push("Project Name is required");
                    }
                    if state.project.metadata.application_name.trim().is_empty() {
                        errors.push("Application Name is required");
                    }
                    if state.project.metadata.version.trim().is_empty() {
                        errors.push("Version is required");
                    }
                    if state.project.metadata.author.trim().is_empty() {
                        errors.push("Author is required");
                    }
                    if state.project.metadata.description.trim().is_empty() {
                        errors.push("Description is required");
                    }
                    if state.project.package_name.trim().is_empty() {
                        errors.push("Package Name is required");
                    }
                    
                    errors
                };
                drop(state);
                
                if !validation_errors.is_empty() {
                    let error_msg = validation_errors.join("\n• ");
                    let alert = AlertDialog::builder()
                        .message("Missing Required Fields")
                        .detail(&format!("Please fill in all required fields:\n• {}", error_msg))
                        .build();
                    alert.show(Some(&window));
                    return;
                }
                
                let buffer = build_log_view.buffer();
                buffer.set_text(""); 
                
                let log_buffer_local = buffer.clone();
                let log = move |msg: &str| {
                    let mut end = log_buffer_local.end_iter();
                    log_buffer_local.insert(&mut end, &format!("{}\n", msg));
                    // Pump event loop to show progress
                    while glib::MainContext::default().iteration(false) {}
                };

                let state = app_state.borrow();
                let output_dir = state.project.metadata.output_directory.clone();
                let package_name = state.project.package_name.clone();
                let files = state.project.files.clone();
                let metadata = state.project.metadata.clone();
                let project = state.project.clone();
                drop(state); 

                let output_path = output_dir.join(&package_name);
                let check_path = output_path.clone(); 

                let window_for_build = window.clone();
                
                let output_dir_for_build = output_dir.clone();
                let output_path_for_build = output_path.clone();
                let log_buffer = buffer.clone();
                
                // Actual Build Work
                let perform_build = Rc::new(move || {
                    let log = |msg: &str| {
                        let mut end = log_buffer.end_iter();
                        log_buffer.insert(&mut end, &format!("-> {}\n", msg));
                        while glib::MainContext::default().iteration(false) {}
                    };

                    log("Starting build process...");

                    // Ensure output directory exists
                    if !output_dir_for_build.exists() {
                         log(&format!("Creating output directory: {:?}", output_dir_for_build));
                         if let Err(e) = std::fs::create_dir_all(&output_dir_for_build) {
                             log(&format!("ERROR: Could not create output directory: {}", e));
                             let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create output directory: {}", e)).build();
                             alert.show(Some(&window_for_build));
                             return;
                         }
                    }
                    
                    // Create unique temporary directory
                    let temp_dir = std::env::temp_dir().join(format!("obision-build-{}", std::process::id()));
                    log(&format!("Creating temporary build directory: {:?}", temp_dir));
                    
                    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                        log(&format!("ERROR: Could not create temp directory: {}", e));
                        let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create temp directory: {}", e)).build();
                        alert.show(Some(&window_for_build));
                        return;
                    }
                    
                    // Create install/ and application/ directories
                    let install_dir = temp_dir.join("install");
                    let application_dir = temp_dir.join("application");
                    
                    if let Err(e) = std::fs::create_dir(&install_dir) {
                        log(&format!("ERROR: Could not create install directory: {}", e));
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create install directory: {}", e)).build();
                        alert.show(Some(&window_for_build));
                        return;
                    }
                    
                    if let Err(e) = std::fs::create_dir(&application_dir) {
                        log(&format!("ERROR: Could not create application directory: {}", e));
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create application directory: {}", e)).build();
                        alert.show(Some(&window_for_build));
                        return;
                    }
                    
                    // === POPULATE INSTALL FOLDER ===
                    log("Populating install/ folder...");
                    
                    // Copy .desktop file to install/
                    if let Some(ref desktop_file) = metadata.desktop_file {
                        if desktop_file.exists() {
                            if let Some(filename) = desktop_file.file_name() {
                                let dest = install_dir.join(filename);
                                log(&format!("Copying: {} -> install/{}", desktop_file.display(), filename.to_string_lossy()));
                                if let Err(e) = std::fs::copy(desktop_file, &dest) {
                                    log(&format!("ERROR copying desktop file: {}", e));
                                }
                            }
                        } else {
                            log("WARNING: Desktop file not found");
                        }
                    } else {
                        log("WARNING: No desktop file specified");
                    }
                    
                    // === POPULATE APPLICATION FOLDER ===
                    log("Populating application/ folder...");
                    
                    for file_entry in &files {
                        if file_entry.source.exists() {
                            let dest = application_dir.join(&file_entry.destination);
                            log(&format!("Copying: {} -> application/{}", file_entry.source.display(), file_entry.destination));
                            
                            // Create parent directories if needed
                            if let Some(parent) = dest.parent() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    log(&format!("ERROR creating directory for {}: {}", file_entry.destination, e));
                                    continue;
                                }
                            }
                            
                            if let Err(e) = std::fs::copy(&file_entry.source, &dest) {
                                log(&format!("ERROR copying file: {}", e));
                            }
                        } else {
                            log(&format!("WARNING: File not found: {}", file_entry.source.display()));
                        }
                    }
                    
                    // === CREATE .LIS ARCHIVE FROM TEMP DIRECTORY ===
                    log(&format!("Creating package file: {:?}", output_path_for_build));
                    let f = match File::create(&output_path_for_build) {
                        Ok(f) => f,
                        Err(e) => {
                            log(&format!("ERROR: Could not create package file: {}", e));
                            let _ = std::fs::remove_dir_all(&temp_dir);
                            let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create package file: {}", e)).build();
                            alert.show(Some(&window_for_build));
                            return;
                        }
                    };
                    
                    let enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
                    let mut tar = tar::Builder::new(enc);
                    
                    log("Adding install/ directory to archive...");
                    if let Err(e) = tar.append_dir_all("install", &install_dir) {
                        log(&format!("ERROR adding install directory: {}", e));
                    }
                    
                    log("Adding application/ directory to archive...");
                    if let Err(e) = tar.append_dir_all("application", &application_dir) {
                        log(&format!("ERROR adding application directory: {}", e));
                    }
                    
                    // Generate comprehensive metadata.toml with all project information
                     log("Generating metadata.toml...");
                     
                     // Convert project files to metadata format
                     let metadata_files: Vec<liblis::metadata::FileEntry> = files.iter().map(|f| {
                         liblis::metadata::FileEntry {
                             source: f.destination.clone(), // In the .lis, it's in application/ folder
                             destination: f.destination.clone(),
                             permissions: f.permissions.clone(),
                         }
                     }).collect();
                     
                     // Convert installer screens to metadata format
                     let metadata_screens: Vec<liblis::metadata::InstallerScreen> = project.installer_screens.iter().map(|s| {
                         liblis::metadata::InstallerScreen {
                             id: s.id.clone(),
                             enabled: s.enabled,
                             order: s.order,
                             custom_content: s.custom_content.clone(),
                         }
                     }).collect();
                     
                     // Create complete metadata structure
                     let full_metadata = liblis::Metadata {
                         package: liblis::metadata::PackageInfo {
                             name: metadata.name.clone(),
                             version: metadata.version.clone(),
                             app_id: format!("com.example.{}", metadata.name.to_lowercase().replace(" ", "-")),
                             description: metadata.description.clone(),
                             author: metadata.author.clone(),
                             application_name: metadata.application_name.clone(),
                             package_name: package_name.clone(),
                             compression_level: project.compression_level,
                         },
                         installation: liblis::metadata::InstallationInfo {
                             prefix_system: "/usr/local".to_string(),
                             prefix_user: "~/.local".to_string(),
                         },
                         desktop: liblis::metadata::DesktopInfo {
                             name: metadata.application_name.clone(),
                             exec: metadata.name.to_lowercase().replace(" ", "-"),
                             icon: String::new(),
                             categories: vec!["Utility".to_string()],
                         },
                         dependencies: liblis::metadata::DependenciesInfo {
                             bundled: vec![],
                         },
                         files: metadata_files,
                         installer_screens: metadata_screens,
                     };
                     
                     // Serialize to TOML
                     let toml_content = match full_metadata.to_toml() {
                         Ok(content) => content,
                         Err(e) => {
                             log(&format!("ERROR serializing metadata: {}", e));
                             let _ = std::fs::remove_dir_all(&temp_dir);
                             let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Error creating metadata: {}", e)).build();
                             alert.show(Some(&window_for_build));
                             return;
                         }
                     };
                     
                     let mut header = tar::Header::new_gnu();
                     header.set_size(toml_content.len() as u64);
                     header.set_mode(0o644);
                     header.set_cksum();
                     
                     if let Err(e) = tar.append_data(&mut header, "metadata.toml", toml_content.as_bytes()) {
                          log(&format!("ERROR adding metadata: {}", e));
                     }

                    log("Finalizing archive...");
                    match tar.finish() {
                        Ok(_) => {
                            log("Cleaning up temporary directory...");
                            let _ = std::fs::remove_dir_all(&temp_dir);
                            log("Build Successful!");
                            let alert = AlertDialog::builder().message("Build Successful").detail(&format!("Package created at {:?}", output_path_for_build)).build();
                            alert.show(Some(&window_for_build));
                        },
                        Err(e) => {
                            log(&format!("ERROR finalizing archive: {}", e));
                            let _ = std::fs::remove_dir_all(&temp_dir);
                            let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Error finalizing archive: {}", e)).build();
                            alert.show(Some(&window_for_build));
                        }
                    }
                });

                if check_path.exists() {
                    log(&format!("File exists at {:?}. Asking for overwrite...", check_path));
                    let alert = AlertDialog::builder()
                        .modal(true)
                        .message("File Exists")
                        .detail(&format!("The file {:?} already exists. Do you want to replace it?", check_path.file_name().unwrap_or_default().to_string_lossy()))
                        .buttons(["Cancel", "Replace"])
                        .default_button(1)
                        .cancel_button(0)
                        .build();
                        
                    let perform_build = perform_build.clone();
                    let window = window.clone();
                    let log_buffer = buffer.clone();
                    
                    alert.choose(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                        if let Ok(res) = result {
                            if res == 1 { // Replace
                                {
                                    let mut end = log_buffer.end_iter();
                                    log_buffer.insert(&mut end, "-> Overwrite confirmed.\n");
                                    while glib::MainContext::default().iteration(false) {}
                                }
                                perform_build();
                            } else {
                                {
                                    let mut end = log_buffer.end_iter();
                                    log_buffer.insert(&mut end, "-> Build cancelled by user.\n");
                                    while glib::MainContext::default().iteration(false) {}
                                }
                            }
                        }
                    });
                } else {
                    perform_build();
                }
            };

            check_unsaved(Rc::new(run_build));
        }
    });

    // Close Request Handler
    {
        let check_unsaved = check_unsaved.clone();
        let app_state = app_state.clone();
        let window = window.clone();
        window.connect_close_request(move |win| {
             let state = app_state.borrow();
             if state.is_modified {
                 drop(state);
                 let check_unsaved = check_unsaved.clone();
                 let win_clone = win.clone();
                 // We must return true (Stop) immediately.
                 // We launch the check. If they proceed, we close manually.
                 check_unsaved(Rc::new(move || {
                     // Force close logic
                     // Since check_unsaved Discard sets is_modified false,
                     // calling close() again should work.
                     win_clone.close();
                 }));
                 return glib::Propagation::Stop;
             }
             glib::Propagation::Proceed
        });
    }

    let app = window.application().unwrap();

    // NEW PROJECT
    if let Some(action) = app.lookup_action("new-project") {
        let simple = action.downcast::<gio::SimpleAction>().unwrap();
        
        let app_state = app_state.clone();
        let update_ui = update_ui.clone();
        let update_title = update_title.clone();
        let content_stack = content_stack.clone();
        let check_unsaved = check_unsaved.clone();
        let update_save_action = update_save_action.clone();
        
        let root_stack = root_stack.clone();
        let window_clone = window.clone();
        
        simple.connect_activate(move |_, _| {
             let app_state = app_state.clone();
             let update_ui = update_ui.clone();
             let update_title = update_title.clone();
             let content_stack = content_stack.clone();
             let root_stack = root_stack.clone();
             let window = window_clone.clone();
             let update_save_action = update_save_action.clone();
             
             check_unsaved(Rc::new(move || {
                 let file_dialog = FileDialog::builder().title("Select .desktop File").modal(true).build();
                 
                 // Add filter for .desktop files
                 let filter = gtk4::FileFilter::new();
                 filter.add_pattern("*.desktop");
                 filter.set_name(Some("Desktop Entry Files"));
                 let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                 filters.append(&filter);
                 file_dialog.set_filters(Some(&filters));
                 
                 let app_state = app_state.clone();
                 let update_ui = update_ui.clone();
                 let update_title = update_title.clone();
                 let update_save_action = update_save_action.clone();
                 let content_stack = content_stack.clone();
                 let root_stack = root_stack.clone();
                 let window_for_error = window.clone();
                 
                 file_dialog.open(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                     if let Ok(file) = result {
                         if let Some(desktop_path) = file.path() {
                             // Parse .desktop file
                             match parse_desktop_file(&desktop_path) {
                                 Ok((app_name, description)) => {
                                     // Get parent directory (should be 'data' folder)
                                     if let Some(data_dir) = desktop_path.parent() {
                                         if let Some(project_root) = data_dir.parent() {
                                             let mut state = app_state.borrow_mut();
                                             state.project = Project::new();
                                             
                                             // Set project root as output directory
                                             state.project.metadata.output_directory = project_root.to_path_buf();
                                             
                                             // Extract project name from folder
                                             if let Some(name) = project_root.file_name() {
                                                 state.project.metadata.name = name.to_string_lossy().to_string();
                                             }
                                             
                                             // Set application name and description from .desktop file
                                             state.project.metadata.application_name = app_name;
                                             state.project.metadata.description = description;
                                             
                                             // Store desktop file path
                                             state.project.metadata.desktop_file = Some(desktop_path.clone());
                                             
                                             state.current_path = None;
                                             state.project.package_name = format!("{}.lis", state.project.metadata.name.to_lowercase().replace(" ", "-"));
                                             state.is_modified = false; 
                                             drop(state);
                                             
                                             update_ui();
                                             update_title();
                                             update_save_action();
                                             root_stack.set_visible_child_name("main_view");
                                             content_stack.set_visible_child_name("configuration");
                                         } else {
                                             let alert = AlertDialog::builder()
                                                 .message("Invalid File Location")
                                                 .detail("The .desktop file must be in a 'data' folder within your project root.")
                                                 .build();
                                             alert.show(Some(&window_for_error));
                                         }
                                     } else {
                                         let alert = AlertDialog::builder()
                                             .message("Invalid File Location")
                                             .detail("Could not determine project structure.")
                                             .build();
                                         alert.show(Some(&window_for_error));
                                     }
                                 },
                                 Err(e) => {
                                     let alert = AlertDialog::builder()
                                         .message("Failed to Parse .desktop File")
                                         .detail(&e)
                                         .build();
                                     alert.show(Some(&window_for_error));
                                 }
                             }
                         }
                     }
                 });
             }));
        });
    }
    
    // OPEN PROJECT
    if let Some(action) = app.lookup_action("open-project") {
        let simple = action.downcast::<gio::SimpleAction>().unwrap();
        let app_state = app_state.clone();
        let update_ui = update_ui.clone();
        let update_title = update_title.clone();
        let update_save_action = update_save_action.clone();
        let root_stack = root_stack.clone();
        let window_clone = window.clone(); // Clone outside
        let content_stack = content_stack.clone(); // Clone outside

        simple.connect_activate(move |_, _| {
             let app_state = app_state.clone();
             let update_ui = update_ui.clone();
             let update_title = update_title.clone();
             let update_save_action = update_save_action.clone();
             let window = window_clone.clone(); // Use clone
             let content_stack = content_stack.clone();
             let root_stack = root_stack.clone();
             
             let update_save_action = update_save_action.clone();
             
             check_unsaved(Rc::new(move || {
                let file_dialog = FileDialog::builder().title("Open Project").modal(true).build();
                let filter = gtk4::FileFilter::new();
                filter.add_pattern("*.lisproj");
                filter.set_name(Some("Obision Project Files"));
                let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                filters.append(&filter);
                file_dialog.set_filters(Some(&filters));
                
                let app_state = app_state.clone();
                let update_ui = update_ui.clone();
                let content_stack = content_stack.clone();
                let root_stack = root_stack.clone(); // Move to closure
                let update_title = update_title.clone(); 
                let update_save_action = update_save_action.clone();

                file_dialog.open(Some(&window), gtk4::gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                             match Project::load_from_file(&path) {
                                 Ok(proj) => {
                                     let mut state = app_state.borrow_mut();
                                     state.project = proj;
                                     state.current_path = Some(path);
                                     state.is_modified = false;
                                     drop(state);
                                     update_ui();
                                     update_ui();
                                     update_title();
                                     update_save_action();
                                     root_stack.set_visible_child_name("main_view"); // Switch view
                                     content_stack.set_visible_child_name("configuration");
                                 },
                                 Err(e) => eprintln!("Error: {}", e),
                             }
                        }
                    }
                });
             }));
        });
    }

    // SAVE Logic


    // SAVE
    if let Some(action) = app.lookup_action("save-project") {
        let simple = action.downcast::<gio::SimpleAction>().unwrap();
        let app_state = app_state.clone();
        let perform_save = perform_save.clone();
        let window_clone = window.clone(); // Fix move
        
        simple.connect_activate(move |_, _| {
             let state = app_state.borrow();
             if let Some(path) = &state.current_path {
                 let path = path.clone();
                 drop(state);
                 let perform_save = perform_save.clone();
                 perform_save(path);
             } else {
                 drop(state);
                 // Save As logic
                 let file_dialog = FileDialog::builder().title("Save As").initial_name("myproject.lisproj").modal(true).build();
                 let filter = gtk4::FileFilter::new();
                 filter.add_pattern("*.lisproj");
                 filter.set_name(Some("Obision Project Files"));
                 let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                 filters.append(&filter);
                 file_dialog.set_filters(Some(&filters));
                 
                 let perform_save = perform_save.clone();
                 file_dialog.save(Some(&window_clone), gtk4::gio::Cancellable::NONE, move |result| {
                     if let Ok(file) = result {
                         if let Some(path) = file.path() {
                             perform_save(path);
                         }
                     }
                 });
             }
        });
    }
    
    // SAVE AS
    if let Some(action) = app.lookup_action("save-as-project") {
        let simple = action.downcast::<gio::SimpleAction>().unwrap();
        let perform_save = perform_save.clone();
        let window_clone = window.clone();
        
        simple.connect_activate(move |_, _| {
             let file_dialog = FileDialog::builder().title("Save As").initial_name("myproject.lisproj").modal(true).build();
             let filter = gtk4::FileFilter::new();
             filter.add_pattern("*.lisproj");
             filter.set_name(Some("Obision Project Files"));
             let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
             filters.append(&filter);
             file_dialog.set_filters(Some(&filters));
             
             let perform_save = perform_save.clone();
             file_dialog.save(Some(&window_clone), gtk4::gio::Cancellable::NONE, move |result| {
                 if let Ok(file) = result {
                     if let Some(path) = file.path() {
                         perform_save(path);
                     }
                 }
             });
        });
    }

    // Initialize UI
    update_title();
    update_ui();
    update_save_action();
    
    // Sidebar nav
    let content_stack_clone = content_stack.clone(); 
    sidebar_list.connect_row_activated(move |_, row| {
        let index = row.index();
        match index {
            0 => content_stack_clone.set_visible_child_name("configuration"),
            1 => content_stack_clone.set_visible_child_name("files"),
            2 => content_stack_clone.set_visible_child_name("screens"),
            3 => content_stack_clone.set_visible_child_name("build"),
            _ => {}
        }
    });

    if let Some(first_row) = sidebar_list.row_at_index(0) {
        sidebar_list.select_row(Some(&first_row));
    }
    
    // == Connect Installer Screen Switches ==
    
    // Helper to update screen enabled state
    let update_screen_state = {
        let app_state = app_state.clone();
        let mark_modified = mark_modified.clone();
        move |screen_id: &str, enabled: bool| {
            let mut state = app_state.borrow_mut();
            if let Some(screen) = state.project.installer_screens.iter_mut().find(|s| s.id == screen_id) {
                screen.enabled = enabled;
                drop(state);
                mark_modified();
            }
        }
    };
    
    // Connect welcome screen switch
    {
        let update_fn = update_screen_state.clone();
        screen_welcome.connect_active_notify(move |switch| {
            update_fn("welcome", switch.is_active());
        });
    }
    
    // Connect license screen switch
    {
        let update_fn = update_screen_state.clone();
        screen_license.connect_active_notify(move |switch| {
            update_fn("license", switch.is_active());
        });
    }
    
    // Connect install_location screen switch
    {
        let update_fn = update_screen_state.clone();
        screen_install_location.connect_active_notify(move |switch| {
            update_fn("install_location", switch.is_active());
        });
    }
    
    // Connect finish screen switch
    {
        let update_fn = update_screen_state.clone();
        screen_finish.connect_active_notify(move |switch| {
            update_fn("finish", switch.is_active());
        });
    }
    
    // Load initial screen states from project
    {
        let state = app_state.borrow();
        for screen in &state.project.installer_screens {
            match screen.id.as_str() {
                "welcome" => screen_welcome.set_active(screen.enabled),
                "license" => screen_license.set_active(screen.enabled),
                "install_location" => screen_install_location.set_active(screen.enabled),
                "finish" => screen_finish.set_active(screen.enabled),
                _ => {}
            }
        }
    }
}

fn show_welcome_screen(window: &adw::ApplicationWindow, root_stack: &gtk4::Stack) {
    let welcome_builder = gtk4::Builder::from_string(include_str!("../data/ui/welcome.ui"));
    let welcome_view: adw::ToolbarView = welcome_builder.object("welcome_view").expect("Could not get welcome_view");
    let new_btn: gtk4::Button = welcome_builder.object("new_project_button").expect("new_project_button");
    let open_btn: gtk4::Button = welcome_builder.object("open_project_button").expect("open_project_button");
    let close_btn: gtk4::Button = welcome_builder.object("close_welcome_button").expect("close_welcome_button");



    
    let window_clone = window.clone();
    new_btn.connect_clicked(move |_| {
        if let Some(app) = window_clone.application() {
            app.activate_action("new-project", None);
        }
    });
    
    let window_clone = window.clone();
    open_btn.connect_clicked(move |_| {
         if let Some(app) = window_clone.application() {
            app.activate_action("open-project", None);
        }
    });
    
    // Close button? Original purpose was to 'close welcome and show main'? 
    // Now welcome IS the main content initially unless replaced.
    // If we are in "single window" mode, closing welcome means closing app?
    // Or just "Start" (Enter empty project).
    // Let's make "Start" -> New Project.
    let window_clone = window.clone();
    close_btn.connect_clicked(move |_| {
        // Just treat as new project or close dialog? 
        // Previously it called `window.close()`.
        window_clone.close();
    });

    root_stack.add_named(&welcome_view, Some("welcome_screen"));
    root_stack.set_visible_child_name("welcome_screen");
}
