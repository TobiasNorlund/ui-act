import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class MyExtensionPreferences extends ExtensionPreferences {

    fillPreferencesWindow(window) {
        // Create a preferences page
        const page = new Adw.PreferencesPage({
            title: 'General',
            icon_name: 'dialog-information-symbolic',
        });
        window.add(page);

        // Create a preferences group
        const group = new Adw.PreferencesGroup({
            title: 'API Keys',
            description: 'Configure API keys'
        });
        page.add(group);

        // Add preference rows
        const anthropicApiKey = new Adw.PasswordEntryRow({
            title: 'Anthropic',
        });
        group.add(anthropicApiKey);

        // Bind to GSettings
        const settings = this.getSettings();
        settings.bind('anthropic-api-key', anthropicApiKey, 'text', Gio.SettingsBindFlags.DEFAULT);
    }
}