#include <adwaita.h>

static void
on_button_clicked_cb (GtkButton *button, gpointer user_data)
{
    gtk_button_set_label (button, "¡Clickeado!");
}

static void
activate_cb (GtkApplication *app)
{
    GtkBuilder *builder;
    GtkWidget *window;
    GtkWidget *button;
    GError *error = NULL;

    /* Load UI from resource */
    builder = gtk_builder_new ();
    if (!gtk_builder_add_from_resource (builder, "/com/obision/example/C/window.ui", &error)) {
        g_critical ("Failed to load UI: %s", error->message);
        g_error_free (error);
        g_object_unref (builder);
        return;
    }

    /* Get window */
    window = GTK_WIDGET (gtk_builder_get_object (builder, "window"));
    gtk_window_set_application (GTK_WINDOW (window), app);

    /* Get button and connect signal */
    button = GTK_WIDGET (gtk_builder_get_object (builder, "primary_button"));
    g_signal_connect (button, "clicked", G_CALLBACK (on_button_clicked_cb), NULL);

    /* Present window */
    gtk_window_present (GTK_WINDOW (window));

    g_object_unref (builder);
}

int
main (int argc, char *argv[])
{
    g_autoptr(AdwApplication) app = NULL;

    app = adw_application_new ("com.obision.example.C", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect (app, "activate", G_CALLBACK (activate_cb), NULL);

    return g_application_run (G_APPLICATION (app), argc, argv);
}
