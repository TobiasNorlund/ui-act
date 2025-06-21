/*
 * extension.js
 *
 * This is the main file for the extension. It's where you'll
 * initialize and enable/disable your extension's functionality.
 */

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import { PopupMenu, PopupMenuItem} from 'resource:///org/gnome/shell/ui/popupMenu.js';
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
        this._selectionRect = null; // Store the selected window's rect
        this.connect('repaint', () => this._onRepaint());
        this.set_size(global.screen_width, global.screen_height);
    }

    setSelection(window) {
        console.log(`setting selection for window: ${window}`);
        if (window) {
            const rect = window.get_frame_rect();
            this._selectionRect = rect;
            this.queue_repaint();
        } else {
            this._selectionRect = null;
            this.queue_repaint();
        }
    }
    
    _onRepaint() {
        const cr = this.get_context();
        cr.setSourceRGBA(0, 0, 0, 0);
        cr.setOperator(Cairo.Operator.SOURCE);
        cr.paint();

        cr.setSourceRGBA(0, 0, 0, 0.5);
        cr.setOperator(Cairo.Operator.OVER);
        cr.paint();

        // Draw the hole if a selection is set
        if (this._selectionRect) {
            cr.setOperator(Cairo.Operator.CLEAR);
            cr.rectangle(
                this._selectionRect.x,
                this._selectionRect.y,
                this._selectionRect.width,
                this._selectionRect.height
            );
            cr.fill();
            cr.setOperator(Cairo.Operator.OVER);
        }
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

        let topBar = new St.BoxLayout();
        const svgPath = extension.path + '/images/uiact_gw.svg';
        const svgFile = Gio.File.new_for_path(svgPath);
        const svgIcon = new St.Icon({
            style_class: 'ui-act-icon',
            gicon: new Gio.FileIcon({ file: svgFile }),
        });
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
        topBar.add_child(new St.Widget({ x_expand: true })); // Spacer
        topBar.add_child(closeButton);
        this.add_child(topBar);

        this.promptInput = new St.Entry({
            style_class: 'prompt-input',
            hint_text: 'Describe a task',
            can_focus: true,
            x_expand: true,
        });
        this.add_child(this.promptInput);
        this.add_child(new St.Widget({ y_expand: true })); // Spacer

        const bottomBar = new St.BoxLayout();
        bottomBar.add_child(new St.Widget({ x_expand: true }));

        // Create the play button
        const runButton = new St.Button({
            style_class: 'run-button',
            //x_align: Clutter.ActorAlign.END,
            child: new St.Icon({
                icon_name: 'media-playback-start-symbolic',
                style_class: 'popup-menu-icon',
            }),
        });
        bottomBar.add_child(runButton);

        // Add the bottom bar to the main container
        this.add_child(bottomBar);

    }
});


export default class UIActExtension extends Extension {
    enable() {
        console.log('Enabling UI Act Extension');

        // Init
        this._modal_grab = null;

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
        this._root = new St.Widget({
            layout_manager: new Clutter.BinLayout(),
            x_expand: true,
            y_expand: true,
            reactive: true,
            visible: false
        });
    
        // Semi-transparent fullscreen background
        this._windowSeletionOverlay = new WindowSelectionOverlay({
            reactive: true,
            can_focus: false,
        });
        this._root.add_child(this._windowSeletionOverlay);
    
        // Foreground white rounded box
        this._launcherUI = new LauncherUI(this);
        this._launcherUI.connect('closed', () => {
            this.hide();
        });
    
        // Position in center of screen
        this._root.add_child(this._launcherUI);
        this._launcherUI.set_x_align(Clutter.ActorAlign.CENTER);
        this._launcherUI.set_y_align(Clutter.ActorAlign.MIDDLE);
    
        // Add the whole thing as chrome
        Main.layoutManager.addChrome(this._root);

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
        if (this._root) {
            this._root.destroy();
            this._root = null;
        }
    }

    show() {
        if (this._root.visible)
            return;

        console.log("Showing UI Act launcher");
        this._root.visible = true;
        this._modal_grab = Main.pushModal(this._root);

        // Update selected window in background
        const workspace = global.workspace_manager.get_active_workspace();
        const stackedWindows = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, workspace);
        const firstWindow = stackedWindows.length > 0 ? stackedWindows[0] : null;
        this._windowSeletionOverlay.setSelection(firstWindow);

        // Add key event handler for Escape
        if (!this._keyPressEventHandler) {
            this._keyPressEventHandler = this._root.connect('key-press-event', (actor, event) => {
                let symbol = event.get_key_symbol();
                if (symbol === Clutter.KEY_Escape && this._root.visible) {
                    this.hide();
                    return Clutter.EVENT_STOP;
                }
                return Clutter.EVENT_PROPAGATE;
            });
        }

        // Focus the prompt input
        if (this._launcherUI && this._launcherUI.promptInput) {
            this._launcherUI.promptInput.grab_key_focus();
        }
    }

    hide() {
        console.log("Hiding UI Act launcher");
        this._root.visible = false;
        if (this._modal_grab)
            Main.popModal(this._modal_grab);

        // Disconnect key event handler
        if (this._keyPressEventHandler) {
            this._root.disconnect(this._keyPressEventHandler);
            this._keyPressEventHandler = null;
        }

        // let workspace = global.workspace_manager.get_active_workspace();
        // let stackedWindows = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, workspace);
        
        // stackedWindows.forEach((window, index) => {
        //     let rect = window.get_frame_rect();
        //     console.log(`${index}: ${window.get_title()} - x:${rect.x} y:${rect.y} w:${rect.width} h:${rect.height}`);
        // });
    }
}
