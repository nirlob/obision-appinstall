#!/usr/bin/env gjs

imports.gi.versions.Gtk = '4.0';
imports.gi.versions.Adw = '1';

const { Gtk, Adw, Gio, GLib } = imports.gi;

class DemoApplication {
    constructor() {
        this.application = new Adw.Application({
            application_id: 'com.obision.example.JavaScript',
            flags: Gio.ApplicationFlags.FLAGS_NONE,
        });

        this.application.connect('activate', this._onActivate.bind(this));
    }

    _onActivate() {

        // Present window
        window.present();
    }

    _onButtonClicked(button) {
        button.set_label('¡Clickeado!');
    }

    run(argv) {
        return this.application.run(argv);
    }
}

const app = new DemoApplication();
app.run(ARGV);
