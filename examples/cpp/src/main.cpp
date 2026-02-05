#include <adwaita.h>
#include <iostream>

extern "C" {

static void activate_cb(GtkApplication* app, gpointer user_data) {
    GtkBuilder* builder = gtk_builder_new_from_resource("/com/obision/example/Cpp/window.ui");
    
    // Get window
    auto window = GTK_WINDOW(gtk_builder_get_object(builder, "window"));
    gtk_window_set_application(window, app);
    
    // Get button and connect signal
    auto button = GTK_BUTTON(gtk_builder_get_object(builder, "primary_button"));
    if (button) {
        g_signal_connect(button, "clicked", G_CALLBACK(+[](GtkButton* btn, gpointer data) {
            gtk_button_set_label(btn, "¡Clickeado!");
        }), nullptr);
    }
    
    // Present window
    gtk_window_present(window);
}

} // extern "C"

int main(int argc, char* argv[]) {
    auto app = adw_application_new("com.obision.example.Cpp", G_APPLICATION_FLAGS_NONE);
    g_signal_connect(app, "activate", G_CALLBACK(activate_cb), nullptr);
    
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    
    return status;
}
