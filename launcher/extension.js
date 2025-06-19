/*
 * extension.js
 *
 * This is the main file for the extension. It's where you'll
 * initialize and enable/disable your extension's functionality.
 */

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import St from 'gi://St';
import GObject from 'gi://GObject';
import GLib from 'gi://GLib';
import Cairo from 'gi://cairo';
import Clutter from 'gi://Clutter';
import Meta from 'gi://Meta';
import Gio from 'gi://Gio';
import Shell from 'gi://Shell';


const ScreenshotButton = GObject.registerClass(
class ScreenshotButton extends PanelMenu.Button {
    _init(extension) {
        super._init(0.0, 'UI Act');

        // Add an icon to the button
        this.add_child(new St.Icon({
            gicon: Gio.icon_new_for_string(extension.path + '/images/uiact_wb.svg'),
            icon_size: 32,
            style_class: 'system-status-icon'
        }));

        // Connect the click event
        //this.connect('button-press-event', () => this._extension.toggleOverlay());
    }
});

const WindowSelectionOverlay = GObject.registerClass(
class WindowSelectionOverlay extends St.DrawingArea {
    _init() {
        super._init();
        
        // Connect the repaint signal
        this.connect('repaint', () => this._onRepaint());
        
        // Set the size
        this.set_size(global.screen_width, global.screen_height);
    }
    
    _onRepaint() {
        const cr = this.get_context();
        
        // Clear the context
        cr.setSourceRGBA(0, 0, 0, 0);
        cr.setOperator(Cairo.Operator.SOURCE);
        cr.paint();
        
        // Set the dark overlay color
        cr.setSourceRGBA(0, 0, 0, 0.5);
        cr.paint();
        
        // Create a hole in the center
        // const holeWidth = 600;
        // const holeHeight = 250;
        // const holeX = 100;
        // const holeY = 100;
        
        // // Use XOR to create the hole (subtract the rectangle from the overlay)
        // cr.setOperator(Cairo.Operator.CLEAR);
        // cr.rectangle(holeX, holeY, holeWidth, holeHeight);
        // cr.fill();
        
        // Restore the operator
        cr.setOperator(Cairo.Operator.OVER);
    }
});


const LauncherUI = GObject.registerClass({
    // We can define signals for our widget.
    // This lets other parts of the code know when the close button was clicked.
    Signals: {
        'closed': {},
    },
},
class LauncherUI extends St.BoxLayout {
    _init(extension, ...params) {
        super._init({
            ...params,
            style_class: 'launch-container',
            vertical: true, // Arrange children top-to-bottom
        });

        // --- Close Button ---
        let topBar = new St.BoxLayout();
        const svgPath = extension.path + '/images/uiact_gw.svg';
        const svgFile = Gio.File.new_for_path(svgPath);
        const svgIcon = new St.Icon({
            style_class: 'ui-act-icon',
            gicon: new Gio.FileIcon({ file: svgFile }),
        });
        let spacer = new St.Widget({ x_expand: true });
        let closeButton = new St.Button({
            style_class: 'close-button',
            y_align: Clutter.ActorAlign.START,
            child: new St.Icon({
                icon_name: 'window-close-symbolic',
                style_class: 'popup-menu-icon',
            }),
        });
        closeButton.connect('clicked', () => {
            this.emit('closed');
        });
        topBar.add_child(svgIcon);
        topBar.add_child(spacer);
        topBar.add_child(closeButton);
        this.add_child(topBar);

        // Store reference to promptInput
        this.promptInput = new St.Entry({
            style_class: 'prompt-input',
            hint_text: 'Describe a task',
            can_focus: true,
            x_expand: true, // Allows it to fill the width of contentBox
        });
        this.add_child(this.promptInput);

        let contentLabel = new St.Label({
            text: 'This is the main content area.',
            style_class: 'content-label',
            x_align: Clutter.ActorAlign.CENTER,
            y_align: Clutter.ActorAlign.CENTER,
            y_expand: true,
        });
        this.add_child(contentLabel);
    }
});


export default class UIActExtension extends Extension {
    enable() {
        console.log('Enabling UI Act Extension');

        // Register Super+space keybinding
        this._settings = this.getSettings("org.gnome.shell.extensions.ui-act");
        this._launchKeybindingKey = 'ui-act-launch';
        Main.wm.addKeybinding(
            this._launchKeybindingKey,
            this._settings,
            Meta.KeyBindingFlags.NONE,
            Shell.ActionMode.NORMAL,
            () => this.show()
        );

        // Fullscreen container with BinLayout for manual positioning
        this.overlay = new St.Widget({
            layout_manager: new Clutter.BinLayout(),
            x_expand: true,
            y_expand: true,
            reactive: false,
            visible: false
        });
    
        // Semi-transparent fullscreen background
        let background = new WindowSelectionOverlay({
            reactive: false,
            can_focus: false,
        });
        this.overlay.add_child(background);
    
        // Foreground white rounded box
        let launchContainer = new LauncherUI(this);
        this._launchContainer = launchContainer;
        // Connect to the 'closed' signal to handle the event
        launchContainer.connect('closed', () => {
            this.hide();
        });
    
        // Position in center of screen
        this.overlay.add_child(launchContainer);
        launchContainer.set_x_align(Clutter.ActorAlign.CENTER);
        launchContainer.set_y_align(Clutter.ActorAlign.MIDDLE);
    
        // Add the whole thing as chrome
        Main.layoutManager.addChrome(this.overlay);

        // Create the button and add it to the top panel
        this._indicator = new ScreenshotButton(this);
        Main.panel.addToStatusArea('ui-act', this._indicator);
    }

    disable() {
        console.log('Disabling UI Act Extension');

        // Remove the keybinding
        if (this._launchKeybindingKey) {
            Main.wm.removeKeybinding(this._launchKeybindingKey);
            this._settings = null;
        }

        // Remove the indicator from the panel
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }

        // Destroy the overlay and its child container
        if (this.overlay) {
            this.overlay.destroy();
            this.overlay = null;
        }
    }

    show() {
        console.log("Showing UI Act launcher");
        this.overlay.visible = true;

        // Add key event handler for Escape
        if (!this._keyPressEventId) {
            this._keyPressEventId = global.stage.connect('key-press-event', (actor, event) => {
                let symbol = event.get_key_symbol();
                if (symbol === Clutter.KEY_Escape && this.overlay.visible) {
                    this.hide();
                    return Clutter.EVENT_STOP;
                }
                return Clutter.EVENT_PROPAGATE;
            });
        }

        // Focus the prompt input
        if (this._launchContainer && this._launchContainer.promptInput) {
            this._launchContainer.promptInput.grab_key_focus();
        }
    }

    hide() {
        console.log("Hiding UI Act launcher");
        this.overlay.visible = false;

        // Disconnect key event handler
        if (this._keyPressEventId) {
            global.stage.disconnect(this._keyPressEventId);
            this._keyPressEventId = null;
        }

        // let workspace = global.workspace_manager.get_active_workspace();
        // let stackedWindows = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, workspace);
        
        // stackedWindows.forEach((window, index) => {
        //     let rect = window.get_frame_rect();
        //     console.log(`${index}: ${window.get_title()} - x:${rect.x} y:${rect.y} w:${rect.width} h:${rect.height}`);
        // });
    }
}
