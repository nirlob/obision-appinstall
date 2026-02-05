use gtk4::prelude::*;
use gtk4::{glib, Application};
use libadwaita as adw;

const APP_ID: &str = "com.obision.appinstall.Installer";

fn main() -> glib::ExitCode {
    // Inicializar libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Crear aplicación
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Cargar la interfaz desde el archivo .ui
    let builder = gtk4::Builder::from_string(include_str!("../data/ui/window.ui"));

    // Obtener la ventana
    let window: adw::ApplicationWindow = builder
        .object("window")
        .expect("Could not get window from UI file");

    window.set_application(Some(app));

    // Obtener widgets
    let package_chooser: gtk4::Button = builder
        .object("package_chooser")
        .expect("Could not get package_chooser");

    let install_button: gtk4::Button = builder
        .object("install_button")
        .expect("Could not get install_button");

    let status_label: gtk4::Label = builder
        .object("status_label")
        .expect("Could not get status_label");

    // Configurar eventos
    let status_label_clone = status_label.clone();
    package_chooser.connect_clicked(move |_| {
        status_label_clone.set_text("Selecting package...");
        // TODO: Implementar selector de archivo .lis
    });

    install_button.connect_clicked(move |_| {
        status_label.set_text("Installing package...");
        // TODO: Implementar instalación
    });

    window.present();
}
