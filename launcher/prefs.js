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

        const generalPrefGroup = new Adw.PreferencesGroup({
            title: 'General'
        });
        page.add(generalPrefGroup);
        
        // Add telemetry toggle directly to the page
        const telemetryRow = new Adw.SwitchRow({
            title: 'Send anonymous usage data',
            subtitle: 'Help improve UI Act by sending anonymous telemetry. No keys, screenshots or prompts are sent.'
        });
        generalPrefGroup.add(telemetryRow);

        // Create API preferences group
        const apiPrefGroup = new Adw.PreferencesGroup({
            title: 'API Keys',
            description: 'Configure API keys'
        });
        page.add(apiPrefGroup);
        const anthropicApiKey = new Adw.PasswordEntryRow({title: 'Anthropic'});
        apiPrefGroup.add(anthropicApiKey);

        // Bind to GSettings
        const settings = this.getSettings();
        settings.bind('anthropic-api-key', anthropicApiKey, 'text', Gio.SettingsBindFlags.DEFAULT);
        settings.bind('telemetry-enabled', telemetryRow, 'active', Gio.SettingsBindFlags.DEFAULT);
    }
}