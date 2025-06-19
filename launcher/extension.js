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


const ScreenshotButton = GObject.registerClass(
class ScreenshotButton extends PanelMenu.Button {
    _init(extension) {
        super._init(0.0, 'Screenshot Overlay');
        this._extension = extension;

        // Add an icon to the button
        this.add_child(new St.Icon({
            gicon: Gio.icon_new_for_string(this._extension.path + '/assets/uiact_wb.svg'),
            icon_size: 32,
            style_class: 'system-status-icon'
        }));

        // Connect the click event
        this.connect('button-press-event', () => this._extension.toggleOverlay());
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


export default class UIActExtension extends Extension {
    enable() {
        console.log('Enabling Screenshot Overlay Extension');

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
        let launchContainer = new St.BoxLayout({
            style_class: 'launch-container',
            vertical: true,
            reactive: true,
            x_expand: false,
            y_expand: false,
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
        console.log('Disabling Screenshot Overlay Extension');

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

    toggleOverlay() {
        console.log(`Toggle visibility to ${!this.overlay.visible}`);
        this.overlay.visible = !this.overlay.visible;

        // let workspace = global.workspace_manager.get_active_workspace();
        // let stackedWindows = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, workspace);
        
        // stackedWindows.forEach((window, index) => {
        //     let rect = window.get_frame_rect();
        //     console.log(`${index}: ${window.get_title()} - x:${rect.x} y:${rect.y} w:${rect.width} h:${rect.height}`);
        // });
    }
}
