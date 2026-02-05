#!/usr/bin/env python3

import gi
import sys

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, Gio

class DemoApplication(Adw.Application):
    def __init__(self):
        super().__init__(application_id='com.obision.example.Python')
        self.connect('activate', self.on_activate)

    def on_activate(self, app):
        # Load UI from file
        builder = Gtk.Builder()
        # Use local path for development
        import os
        ui_file = os.path.join(os.path.dirname(__file__), '..', 'data', 'ui', 'window.ui')
        builder.add_from_file(ui_file)

        # Get window
        window = builder.get_object('window')
        window.set_application(app)

        # Get button and connect signal
        button = builder.get_object('primary_button')
        if button:
            button.connect('clicked', self.on_button_clicked)

        # Present window
        window.present()

    def on_button_clicked(self, button):
        button.set_label("¡Clickeado!")

if __name__ == '__main__':
    app = DemoApplication()
    sys.exit(app.run(sys.argv))
