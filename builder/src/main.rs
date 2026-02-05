use gtk4::prelude::*;
use gtk4::{glib, gio, Application, FileDialog, AlertDialog};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs::File;
use libadwaita as adw;
use adw::prelude::AdwApplicationWindowExt;
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
    let project_version_entry: adw::EntryRow = builder.object("project_version_entry").expect("Could not get project_version_entry");
    let project_author_entry: adw::EntryRow = builder.object("project_author_entry").expect("Could not get project_author_entry");
    let package_name_entry: adw::EntryRow = builder.object("package_name_entry").expect("Could not get package_name_entry");
    
    let files_group: adw::PreferencesGroup = builder.object("files_list_group").expect("Could not get files_list_group");
    let add_file_row: adw::ActionRow = builder.object("add_file_row").expect("Could not get add_file_row");
    
    let output_dir_row: adw::ActionRow = builder.object("output_dir_row").expect("Could not get output_dir_row");
    let output_dir_button: gtk4::Button = builder.object("output_dir_button").expect("output_dir_button");
    let build_package_button: gtk4::Button = builder.object("build_package_button").expect("build_package_button");
    let build_log_view: gtk4::TextView = builder.object("build_log_view").expect("build_log_view");

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
        let ver_entry = project_version_entry.clone();
        let auth_entry = project_author_entry.clone();
        let pkg_entry = package_name_entry.clone();
        let output_row = output_dir_row.clone();
        
        move || {
            let (name, ver, auth, pkg, out_dir) = {
                let state = app_state.borrow();
                (
                    state.project.metadata.name.clone(),
                    state.project.metadata.version.clone(),
                    state.project.metadata.author.clone(),
                    state.project.package_name.clone(),
                    state.project.metadata.output_directory.clone(),
                )
            };
            
            name_entry.set_text(&name);
            ver_entry.set_text(&ver);
            auth_entry.set_text(&auth);
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
        (&package_name_entry, 3)
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

                    // Ensure directory exists
                    if !output_dir_for_build.exists() {
                         log(&format!("Creating directory: {:?}", output_dir_for_build));
                         let _ = std::fs::create_dir_all(&output_dir_for_build);
                    }
                    
                    log(&format!("Creating package file: {:?}", output_path_for_build));
                    let f = match File::create(&output_path_for_build) {
                        Ok(f) => f,
                        Err(e) => {
                            log(&format!("ERROR: Could not create file: {}", e));
                            let alert = AlertDialog::builder().message("Build Failed").detail(&format!("Could not create file: {}", e)).build();
                            alert.show(Some(&window_for_build));
                            return;
                        }
                    };
                    
                    let enc = flate2::write::GzEncoder::new(f, flate2::Compression::default());
                    let mut tar = tar::Builder::new(enc);
                    
                    // 1. Add binary/files
                    log("Adding files to archive...");
                    for file_entry in &files {
                        if file_entry.source.exists() {
                            let dest = format!("binary/{}", file_entry.destination); 
                            log(&format!("Adding: {} -> {}", file_entry.source.display(), dest));
                            if let Err(e) = tar.append_path_with_name(&file_entry.source, &dest) {
                                log(&format!("ERROR adding file: {}", e));
                                eprintln!("Failed to add file {:?}: {}", file_entry.source, e);
                            }
                        } else {
                            log(&format!("WARNING: File not found: {}", file_entry.source.display()));
                        }
                    }
                    
                    // 2. Add metadata.toml
                     log("Generating metadata.toml...");
                     let toml_content = format!(
                         r#"[package]
name = "{}"
version = "{}"
app_id = "com.example.{}"
description = "{}"

[installation]
target_dir_system = "/opt/{}"
target_dir_user = "~/.local/share/{}"

[desktop]
name = "{}"
exec = "{}"
icon = ""
categories = ["Utility"]

[dependencies]
bundled = []
"#,
                        metadata.name,
                        metadata.version,
                        metadata.name.to_lowercase().replace(" ", "-"),
                        metadata.description,
                        metadata.name.to_lowercase().replace(" ", "-"),
                        metadata.name.to_lowercase().replace(" ", "-"),
                        metadata.name,
                        metadata.name.to_lowercase().replace(" ", "-"),
                    );
                    
                    let mut header = tar::Header::new_gnu();
                    header.set_size(toml_content.len() as u64);
                    header.set_mode(0o644);
                    header.set_cksum();
                    
                    if let Err(e) = tar.append_data(&mut header, "metadata.toml", toml_content.as_bytes()) {
                         log(&format!("ERROR adding metadata: {}", e));
                         eprintln!("Failed to add metadata: {}", e);
                    }

                    log("Finalizing archive...");
                    match tar.finish() {
                        Ok(_) => {
                            log("Build Successful!");
                            let alert = AlertDialog::builder().message("Build Successful").detail(&format!("Package created at {:?}", output_path_for_build)).build();
                            alert.show(Some(&window_for_build));
                        },
                        Err(e) => {
                            log(&format!("ERROR finalizing archive: {}", e));
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
        
        simple.connect_activate(move |_, _| {
             let app_state = app_state.clone();
             let update_ui = update_ui.clone();
             let update_title = update_title.clone();
             let content_stack = content_stack.clone();
             let root_stack = root_stack.clone();
             
             let update_save_action = update_save_action.clone();
             check_unsaved(Rc::new(move || {
                 let mut state = app_state.borrow_mut();
                 state.project = Project::new();
                 state.current_path = None;
                 state.is_modified = false;
                 drop(state);
                 
                 update_ui();
                 update_title();
                 update_save_action();
                 root_stack.set_visible_child_name("main_view"); // Switch view
                 content_stack.set_visible_child_name("configuration");
                 
                 // Note: we don't show welcome screen here as per standard Ctrl+N behavior
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
