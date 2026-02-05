use gtk4::prelude::*;
use gtk4::{glib, gio, Application, FileDialog};
use libadwaita as adw;
use adw::prelude::AdwApplicationWindowExt;
// use adw::prelude::AdwWindowExt; // Displayed warning, removing

mod project;
use project::Project;

const APP_ID: &str = "com.obision.appinstall.Builder";

fn main() -> glib::ExitCode {
    // Inicializar libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Crear aplicación
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Register actions
    app.connect_startup(|app| {
        setup_actions(app);
    });

    app.run()
}

fn setup_actions(app: &Application) {
    // New Project action
    let new_project_action = gio::SimpleAction::new("new-project", None);
    app.add_action(&new_project_action);
    
    // Open Project action
    let open_project_action = gio::SimpleAction::new("open-project", None);
    app.add_action(&open_project_action);
    
    // Save Project action
    let save_project_action = gio::SimpleAction::new("save-project", None);
    app.add_action(&save_project_action);
    
    // About action
    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(|_, _| {
        println!("About clicked");
        // TODO: Show about dialog
    });
    app.add_action(&about_action);
}

fn build_ui(app: &Application) {
    // Load main window
    let window_builder = gtk4::Builder::from_string(include_str!("../data/ui/window.ui"));
    let window: adw::ApplicationWindow = window_builder
        .object("window")
        .expect("Could not get window from UI file");
    
    window.set_application(Some(app));

    // Setup window interaction
    setup_window_interaction(&window_builder);

    // Show welcome screen first
    show_welcome_screen(&window);
    
    // Present the window
    window.present();
}

fn setup_window_interaction(builder: &gtk4::Builder) {
    let content_stack: gtk4::Stack = builder
        .object("content_stack")
        .expect("Could not get content_stack");
    
    let sidebar_list: gtk4::ListBox = builder
        .object("sidebar_list")
        .expect("Could not get sidebar_list");
    
    let content_header: adw::HeaderBar = builder
        .object("content_header")
        .expect("Could not get content_header");

    // Clone for closures
    let content_stack_clone = content_stack.clone();
    let content_header_clone = content_header.clone();

    // Setup sidebar navigation
    sidebar_list.connect_row_activated(move |_, row| {
        let index = row.index();
        match index {
            0 => {
                content_stack_clone.set_visible_child_name("configuration");
                content_header_clone.set_title_widget(Some(&adw::WindowTitle::new("Configuration", "")));
            }
            1 => {
                content_stack_clone.set_visible_child_name("files");
                content_header_clone.set_title_widget(Some(&adw::WindowTitle::new("Files", "")));
            }
            2 => {
                content_stack_clone.set_visible_child_name("screens");
                content_header_clone.set_title_widget(Some(&adw::WindowTitle::new("Installer Screens", "")));
            }
            3 => {
                content_stack_clone.set_visible_child_name("build");
                content_header_clone.set_title_widget(Some(&adw::WindowTitle::new("Build Package", "")));
            }
            _ => {}
        }
    });

    // Select first row by default (initially)
    if let Some(first_row) = sidebar_list.row_at_index(0) {
        sidebar_list.select_row(Some(&first_row));
    }

    // Get EntryRows for project data (for saving)
    let project_name_entry: adw::EntryRow = builder
        .object("project_name_entry")
        .expect("Could not get project_name_entry");
    let project_version_entry: adw::EntryRow = builder
        .object("project_version_entry")
        .expect("Could not get project_version_entry");
    let project_author_entry: adw::EntryRow = builder
        .object("project_author_entry")
        .expect("Could not get project_author_entry");
    let package_name_entry: adw::EntryRow = builder
        .object("package_name_entry")
        .expect("Could not get package_name_entry");

    // Connect Menu Actions
    let window = builder.object::<adw::ApplicationWindow>("window").expect("Could not get window");
    
    if let Some(app) = window.application() {
        // New Project
        let window_clone = window.clone();
        if let Some(action) = app.lookup_action("new-project") {
            if let Some(simple_action) = action.downcast_ref::<gio::SimpleAction>() {
                simple_action.connect_activate(move |_, _| {
                   show_welcome_screen(&window_clone);
                });
            }
        }

        // Open Project
        let window_clone = window.clone();
        if let Some(action) = app.lookup_action("open-project") {
            if let Some(simple_action) = action.downcast_ref::<gio::SimpleAction>() {
                simple_action.connect_activate(move |_, _| {
                    open_project_dialog(&window_clone);
                });
            }
        }

        // Save Project
        let window_clone = window.clone();
        if let Some(action) = app.lookup_action("save-project") {
            if let Some(simple_action) = action.downcast_ref::<gio::SimpleAction>() {
                // Remove existing handlers to avoid duplicates (optional but good practice if we were using a persisted app action)
                // However, since we re-create the action or just add a signal, GSimpleAction allows multiple signals.
                // NOTE: This might accumulate signals if we open multiple windows, but this is a single window app for now.
                
                // Clone widgets to move into closure
                let name_entry = project_name_entry.clone();
                let version_entry = project_version_entry.clone();
                let author_entry = project_author_entry.clone();
                let pkg_entry = package_name_entry.clone();
                
                simple_action.connect_activate(move |_, _| {
                    println!("Save project action triggered");
                    
                    // Collect data from UI
                    let mut project = Project::new();
                    project.metadata.name = name_entry.text().to_string();
                    project.metadata.version = version_entry.text().to_string();
                    project.metadata.author = author_entry.text().to_string();
                    project.package_name = pkg_entry.text().to_string();
                    
                    // Create FileDialog for saving
                    let file_dialog = FileDialog::builder()
                        .title("Save Project")
                        .initial_name("myproject.lisproj")
                        .modal(true)
                        .build();
                        
                    let filter = gtk4::FileFilter::new();
                    filter.add_pattern("*.lisproj");
                    filter.set_name(Some("Obision Project Files"));
                    
                    let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
                    filters.append(&filter);
                    file_dialog.set_filters(Some(&filters));
                    
                    let project_clone = project.clone();
                    file_dialog.save(Some(&window_clone), gtk4::gio::Cancellable::NONE, move |result| {
                        match result {
                            Ok(file) => {
                                if let Some(path) = file.path() {
                                    if let Err(e) = project_clone.save_to_file(&path) {
                                        eprintln!("Error saving project: {}", e);
                                    } else {
                                        println!("Project saved successfully to {:?}", path);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Save canceled or error: {}", e);
                            }
                        }
                    });
                });
            }
        }
    }
}

fn show_welcome_screen(window: &adw::ApplicationWindow) {
    // Load welcome view
    let welcome_builder = gtk4::Builder::from_string(include_str!("../data/ui/welcome.ui"));
    let welcome_view: adw::ToolbarView = welcome_builder
        .object("welcome_view")
        .expect("Could not get welcome_view");

    // Get buttons
    let new_project_button: gtk4::Button = welcome_builder
        .object("new_project_button")
        .expect("Could not get new_project_button");

    let open_project_button: gtk4::Button = welcome_builder
        .object("open_project_button")
        .expect("Could not get open_project_button");

    let close_welcome_button: gtk4::Button = welcome_builder
        .object("close_welcome_button")
        .expect("Could not get close_welcome_button");

    // Connect events
    let window_clone = window.clone();
    new_project_button.connect_clicked(move |_| {
        println!("New Project clicked");
        create_new_project(&window_clone);
    });

    let window_clone = window.clone();
    open_project_button.connect_clicked(move |_| {
        open_project_dialog(&window_clone);
    });

    let window_clone = window.clone();
    close_welcome_button.connect_clicked(move |_| {
        window_clone.close();
    });

    // Replace window content with welcome view
    window.set_content(Some(&welcome_view));
}

fn create_new_project(window: &adw::ApplicationWindow) {
    // Create a new empty project
    let _project = Project::new();
    
    // Load the main window UI (sidebar interface)
    let new_builder = gtk4::Builder::from_string(include_str!("../data/ui/window.ui"));
    let temp_window: adw::ApplicationWindow = new_builder
        .object("window")
        .expect("Could not get window");
    
    // Setup interactions for the NEW builder instance
    setup_window_interaction(&new_builder);

    if let Some(content) = temp_window.content() {
        content.unparent();
        window.set_content(Some(&content));
    }
    
    // Explicitly re-trigger selection to ensure UI is consistent
    if let Some(sidebar_list) = new_builder.object::<gtk4::ListBox>("sidebar_list") {
        if let Some(first_row) = sidebar_list.row_at_index(0) {
            sidebar_list.select_row(Some(&first_row));
            sidebar_list.emit_by_name::<()>("row-activated", &[&first_row]);
        }
    }
    
    println!("New project created, showing main interface");
}

fn open_project_dialog(window: &adw::ApplicationWindow) {
    // Create file chooser dialog
    let file_dialog = FileDialog::builder()
        .title("Open Project")
        .modal(true)
        .build();
    
    // Create filter for .lisproj files
    let filter = gtk4::FileFilter::new();
    filter.add_pattern("*.lisproj");
    filter.set_name(Some("Obision Project Files"));
    
    let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);
    file_dialog.set_filters(Some(&filters));
    
    let window_clone = window.clone();
    file_dialog.open(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                println!("Opening project: {:?}", path);
                // TODO: Load project and show main interface
                match Project::load_from_file(&path) {
                    Ok(_project) => {
                        // Load the main window UI
                        let new_builder = gtk4::Builder::from_string(include_str!("../data/ui/window.ui"));
                        
                        // Setup interactions for the NEW builder instance
                        setup_window_interaction(&new_builder);

                        if let Some(temp_window) = new_builder.object::<adw::ApplicationWindow>("window") {
                            if let Some(content) = temp_window.content() {
                                content.unparent();
                                window_clone.set_content(Some(&content));
                            }
                        }
                        
                        // Explicitly re-trigger selection
                        if let Some(sidebar_list) = new_builder.object::<gtk4::ListBox>("sidebar_list") {
                            if let Some(first_row) = sidebar_list.row_at_index(0) {
                                sidebar_list.select_row(Some(&first_row));
                                sidebar_list.emit_by_name::<()>("row-activated", &[&first_row]);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load project: {}", e);
                    }
                }
            }
        }
    });
}
