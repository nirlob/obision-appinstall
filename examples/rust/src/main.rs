use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;

fn main() {
    // Inicializar libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Crear la aplicación
    let app = adw::Application::builder()
        .application_id("com.obision.example.Rust")
        .build();

    app.connect_activate(build_ui);

    // Ejecutar la aplicación
    app.run();
}

fn build_ui(app: &adw::Application) {
    // Cargar el archivo UI
    // UI string
    let ui = include_str!("../data/ui/window.ui");
    
    let builder = gtk4::Builder::from_string(ui);
    
    // Obtener la ventana principal del builder
    let window: adw::ApplicationWindow = builder
        .object("window")
        .expect("No se pudo obtener la ventana desde el archivo .ui");
    
    window.set_application(Some(app));
    
    // Obtener referencias a los widgets definidos en el archivo .ui
    let primary_button: gtk4::Button = builder
        .object("primary_button")
        .expect("No se pudo obtener el botón principal");
    
    let dark_mode_switch: gtk4::Switch = builder
        .object("dark_mode_switch")
        .expect("No se pudo obtener el switch de modo oscuro");
    
    // Conectar señales
    primary_button.connect_clicked(|button| {
        button.set_label("¡Clickeado!");
    });
    
    dark_mode_switch.connect_state_set(move |_, is_active| {
        let style_manager = adw::StyleManager::default();
        if is_active {
            style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
        } else {
            style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
        }
        glib::Propagation::Proceed
    });

    // Mostrar la ventana
    window.present();
}
