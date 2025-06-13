import subprocess
from typing import List
from evdev import UInput, ecodes as e, AbsInfo
import time
import re
import base64
from PIL import Image, ImageGrab
import io
from Xlib import X, display, Xatom
from Xlib.error import XError
from Xlib.protocol import event


def get_screen_resolution():
    try:
        # Run xdpyinfo and parse dimensions line
        xdpyinfo = subprocess.run(['xdpyinfo'], capture_output=True, text=True)
        match = re.search(r'dimensions:\s+(\d+)x(\d+)', xdpyinfo.stdout)
        if match:
            width = int(match.group(1))
            height = int(match.group(2))
            return width, height
        else:
            raise RuntimeError("Could not detect screen resolution")
    except subprocess.SubprocessError as e:
        raise RuntimeError("Could not detect screen resolution") from e


def get_device_id(device_name: str, is_exact: bool = True) -> int | None:
    try:
        output = subprocess.check_output(['xinput', 'list', '--short'], text=True)
    except subprocess.CalledProcessError as e:
        raise RuntimeError("Failed to run xinput") from e

    # Match lines like: ↳ CoX Mouse                            id=12   [slave pointer  (2)]
    for line in output.splitlines():
        if (is_exact and device_name in line.split('\t')[0]) or (not is_exact and device_name in line):
            match = re.search(r'id=(\d+)', line)
            if match:
                return int(match.group(1))

    return None


def get_property(window, display, prop_name, prop_type=None):
    atom = display.intern_atom(prop_name)
    prop = window.get_full_property(atom, prop_type or X.AnyPropertyType)
    return prop.value if prop else None


def get_frame_extents(window, display):
    atom = display.intern_atom('_NET_FRAME_EXTENTS')
    prop = window.get_full_property(atom, X.AnyPropertyType)
    if prop and len(prop.value) == 4:
        left, right, top, bottom = prop.value
        return left, right, top, bottom
    return 0, 0, 0, 0  # fallback if not available


def get_visible_windows():
    """Get a list of all visible windows with their properties."""
    try:
        # Initialize X display
        d = display.Display()
        root = d.screen().root
        
        # Get the _NET_CLIENT_LIST property (list of top-level managed windows)
        client_list = root.get_full_property(
            d.intern_atom('_NET_CLIENT_LIST'), 
            X.AnyPropertyType
        )
        
        if not client_list:
            return []
        
        windows = []
        for win_id in client_list.value:
            try:
                window = d.create_resource_object('window', win_id)
                # Get window geometry and name
                try:
                    # Get the window's position relative to its parent
                    geom = window.get_geometry()
                    
                    # Get the window's absolute position by traversing up the hierarchy
                    abs_x = geom.x
                    abs_y = geom.y
                    parent = window.query_tree().parent
                    while parent.id != root.id:
                        parent_geom = parent.get_geometry()
                        abs_x += parent_geom.x
                        abs_y += parent_geom.y
                        parent = parent.query_tree().parent
                    
                    name = get_property(window, d, '_NET_WM_NAME') or get_property(window, d, 'WM_NAME')
                    # Check if window is minimized
                    states = get_property(window, d, '_NET_WM_STATE')
                    if states and d.intern_atom('_NET_WM_STATE_HIDDEN') in states:
                        continue
                except:
                    continue
                # Skip windows without names
                if not name:
                    continue
                windows.append({
                    'id': win_id,
                    'name': name.decode('utf-8') if isinstance(name, bytes) else name,
                    'x': abs_x,
                    'y': abs_y,
                    'width': geom.width,
                    'height': geom.height
                })
            except XError:
                continue
        return windows
    except XError:
        return []


def select_window():
    """List all visible windows and let the user select one."""
    windows = get_visible_windows()
    if not windows:
        print("No visible windows found!")
        return None
    
    print("\nAvailable windows:")
    for i, window in enumerate(windows, 1):
        print(f"{i}. {window['name']} ({window['width']}x{window['height']}) at position ({window['x']},{window['y']})")
    
    while True:
        try:
            choice = input("\nSelect a window (number) or 'q' to quit: ")
            if choice.lower() == 'q':
                return None
            
            index = int(choice) - 1
            if 0 <= index < len(windows):
                return windows[index]
            else:
                print("Invalid selection. Please try again.")
        except ValueError:
            print("Please enter a valid number.")


def get_window_info(window_id):
    """Get window information including position and size."""
    d = display.Display()
    window = d.create_resource_object('window', window_id)
    try:
        # Get the window's position relative to its parent
        geom = window.get_geometry()
        
        # Get the window's absolute position by traversing up the hierarchy
        abs_x = geom.x
        abs_y = geom.y
        parent = window.query_tree().parent
        root = d.screen().root
        while parent.id != root.id:
            parent_geom = parent.get_geometry()
            abs_x += parent_geom.x
            abs_y += parent_geom.y
            parent = parent.query_tree().parent
        
        return {
            'x': abs_x,
            'y': abs_y,
            'width': geom.width,
            'height': geom.height
        }
    except XError:
        raise RuntimeError(f"Could not get window information for window {window_id}")


def set_window_always_on_top(window_id: int, always_on_top: bool = True) -> None:
    """Set a window to be always on top or not using _NET_WM_STATE client message.
    
    Args:
        window_id: The X11 window ID
        always_on_top: Whether the window should be always on top
    """
    d = display.Display()
    window = d.create_resource_object('window', window_id)
    root = d.screen().root

    state_atom = d.intern_atom('_NET_WM_STATE')
    above_atom = d.intern_atom('_NET_WM_STATE_ABOVE')

    if always_on_top:
        action = 1  # _NET_WM_STATE_ADD
    else:
        action = 0  # _NET_WM_STATE_REMOVE

    ev = event.ClientMessage(
        window=window,
        client_type=state_atom,
        data=(
            32,
            [
                action,
                above_atom,
                0,  # No second property
                1,  # Source indication for normal apps
                0,
            ],
        ),
    )
    
    root.send_event(ev, event_mask=X.SubstructureRedirectMask | X.SubstructureNotifyMask)
    d.sync()


class MPXEnvironment:
    def __init__(self):
        self.width, self.height = get_screen_resolution()

    def __enter__(self):
        # Create a virtual mouse device
        screen_width, screen_height = get_screen_resolution()
        mouse_capabilities = {
            e.EV_ABS: [
                (e.ABS_X, AbsInfo(0, 0, screen_width - 1, 0, 0, 0)),
                (e.ABS_Y, AbsInfo(0, 0, screen_height - 1, 0, 0, 0)),
            ],
            e.EV_KEY: [e.BTN_LEFT, e.BTN_RIGHT],
            e.EV_REL: [e.REL_WHEEL, e.REL_HWHEEL],  # Add scroll wheel capabilities
        }
        self.mouse_ui = UInput(mouse_capabilities, name="CoX Mouse Device")
        mouse_id = get_device_id("CoX Mouse Device")
        if mouse_id is None:
            raise RuntimeError("Could not find device ID for 'CoX Mouse Device'")

        # Create a virtual keyboard device
        keyboard_capabilities = {
            e.EV_KEY: [
                e.KEY_A, e.KEY_B, e.KEY_C, e.KEY_D, e.KEY_E, e.KEY_F, e.KEY_G, e.KEY_H, e.KEY_I, e.KEY_J,
                e.KEY_K, e.KEY_L, e.KEY_M, e.KEY_N, e.KEY_O, e.KEY_P, e.KEY_Q, e.KEY_R, e.KEY_S, e.KEY_T,
                e.KEY_U, e.KEY_V, e.KEY_W, e.KEY_X, e.KEY_Y, e.KEY_Z,
                e.KEY_1, e.KEY_2, e.KEY_3, e.KEY_4, e.KEY_5, e.KEY_6, e.KEY_7, e.KEY_8, e.KEY_9, e.KEY_0,
                e.KEY_SPACE, e.KEY_ENTER, e.KEY_BACKSPACE, e.KEY_TAB, e.KEY_ESC,
                e.KEY_LEFTSHIFT, e.KEY_RIGHTSHIFT, e.KEY_LEFTCTRL, e.KEY_RIGHTCTRL,
                e.KEY_LEFTALT, e.KEY_RIGHTALT, e.KEY_LEFTMETA, e.KEY_RIGHTMETA,
                e.KEY_MINUS, e.KEY_EQUAL, e.KEY_LEFTBRACE, e.KEY_RIGHTBRACE,
                e.KEY_SEMICOLON, e.KEY_APOSTROPHE, e.KEY_GRAVE, e.KEY_BACKSLASH,
                e.KEY_COMMA, e.KEY_DOT, e.KEY_SLASH
            ]
        }
        self.keyboard_ui = UInput(keyboard_capabilities, name="CoX Keyboard Device")
        keyboard_id = get_device_id("CoX Keyboard Device")
        if keyboard_id is None:
            raise RuntimeError("Could not find device ID for 'CoX Keyboard Device'")

        # Create a new MPX master device
        result = subprocess.run(['xinput', 'create-master', 'CoX'])
        if result.returncode != 0:
            raise RuntimeError("Failed to create MPX master device")
        self.master_mouse_id = get_device_id("CoX pointer")
        self.master_keyboard_id = get_device_id("CoX keyboard")
        
        # Attach the virtual devices to the MPX master device
        result = subprocess.run(['xinput', 'reattach', str(mouse_id), str(self.master_mouse_id)])
        if result.returncode != 0:
            raise RuntimeError("Failed to attach virtual mouse to MPX master device")
        
        # Seems necessary to avoid very erradic behavior
        time.sleep(0.5)
        
        result = subprocess.run(['xinput', 'reattach', str(keyboard_id), str(self.master_keyboard_id)])
        if result.returncode != 0:
            raise RuntimeError("Failed to attach virtual keyboard to MPX master device")
        
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        # Remove the virtual devices
        print("Closing virtual devices")
        self.mouse_ui.close()
        self.keyboard_ui.close()

        time.sleep(0.1)

        # Remove the MPX master device
        print("Removing MPX master device")
        result = subprocess.run(['xinput', 'remove-master', str(self.master_mouse_id)])
        if result.returncode != 0:
            raise RuntimeError("Failed to remove MPX master device")


    def move_to(self, x, y):
        self.mouse_ui.write(e.EV_ABS, e.ABS_X, x)
        self.mouse_ui.write(e.EV_ABS, e.ABS_Y, y)
        self.mouse_ui.syn()
        print(f"Moved to {x}, {y}")

    def left_click(self):
        self.mouse_ui.write(e.EV_KEY, e.BTN_LEFT, 1)
        self.mouse_ui.syn()
        time.sleep(0.1)
        self.mouse_ui.write(e.EV_KEY, e.BTN_LEFT, 0)
        self.mouse_ui.syn()

    def right_click(self):
        self.mouse_ui.write(e.EV_KEY, e.BTN_RIGHT, 1)
        self.mouse_ui.syn()
        time.sleep(0.1)
        self.mouse_ui.write(e.EV_KEY, e.BTN_RIGHT, 0)
        self.mouse_ui.syn()

    def double_click(self):
        self.left_click()
        time.sleep(0.1)
        self.left_click()

    def _keypress(self, key_code):
        """Press and release a key."""
        self.keyboard_ui.write(e.EV_KEY, key_code, 1)
        self.keyboard_ui.syn()
        time.sleep(0.05)
        self.keyboard_ui.write(e.EV_KEY, key_code, 0)
        self.keyboard_ui.syn()

    def keypress(self, keys: List[str]):
        """Press and release a sequence of keys.
        
        Args:
            keys: A list of key names (e.g., ["ctrl", "c"] for Ctrl+C)
        """
        key_mapping = {
            "ctrl": e.KEY_LEFTCTRL,
            "alt": e.KEY_LEFTALT,
            "shift": e.KEY_LEFTSHIFT,
            "super": e.KEY_LEFTMETA,
            "tab": e.KEY_TAB,
            "enter": e.KEY_ENTER,
            "space": e.KEY_SPACE,
            "backspace": e.KEY_BACKSPACE,
            "escape": e.KEY_ESC,
            "delete": e.KEY_DELETE,
            "home": e.KEY_HOME,
            "end": e.KEY_END,
            "pageup": e.KEY_PAGEUP,
            "pagedown": e.KEY_PAGEDOWN,
            "up": e.KEY_UP,
            "down": e.KEY_DOWN,
            "left": e.KEY_LEFT,
            "right": e.KEY_RIGHT,
        }
        
        # Track which modifier keys are pressed
        pressed_keys = []
        
        try:
            for key in keys:
                key = key.lower()
                
                # Handle letter/number keys
                if len(key) == 1 and (key.isalnum() or key in ".,;'[]\\-=/"):
                    if key.isalpha():
                        key_code = getattr(e, f'KEY_{key.upper()}')
                    elif key.isdigit():
                        key_code = getattr(e, f'KEY_{key}')
                    else:
                        # Handle special characters
                        special_chars = {
                            ".": e.KEY_DOT,
                            ",": e.KEY_COMMA,
                            ";": e.KEY_SEMICOLON,
                            "'": e.KEY_APOSTROPHE,
                            "[": e.KEY_LEFTBRACE,
                            "]": e.KEY_RIGHTBRACE,
                            "\\": e.KEY_BACKSLASH,
                            "-": e.KEY_MINUS,
                            "=": e.KEY_EQUAL,
                            "/": e.KEY_SLASH,
                        }
                        key_code = special_chars.get(key)
                    
                    if key_code:
                        self.keyboard_ui.write(e.EV_KEY, key_code, 1)
                        self.keyboard_ui.syn()
                        pressed_keys.append(key_code)
                
                # Handle modifier and special keys
                elif key in key_mapping:
                    key_code = key_mapping[key]
                    self.keyboard_ui.write(e.EV_KEY, key_code, 1)
                    self.keyboard_ui.syn()
                    pressed_keys.append(key_code)
                
                # Handle function keys (F1-F12)
                elif key.startswith('f') and key[1:].isdigit() and 1 <= int(key[1:]) <= 12:
                    key_code = getattr(e, f'KEY_F{key[1:]}')
                    self.keyboard_ui.write(e.EV_KEY, key_code, 1)
                    self.keyboard_ui.syn()
                    pressed_keys.append(key_code)
                
                else:
                    print(f"WARNING: Unhandled key: {key}")
            
            # Small delay while keys are pressed
            time.sleep(0.1)
            
        finally:
            # Release all keys in reverse order
            for key_code in reversed(pressed_keys):
                self.keyboard_ui.write(e.EV_KEY, key_code, 0)
                self.keyboard_ui.syn()
                time.sleep(0.05)

    def click(self, x: int, y: int, button: str = "left") -> None:
        if button == "left":
            self.move_to(x, y)
            self.left_click()
        elif button == "right":
            self.move_to(x, y)
            self.right_click()
        else:
            raise ValueError(f"Invalid button: {button}")

    def type(self, text):
        """Type a string of text."""
        for char in text:
            if char.isupper():
                # Press shift for uppercase letters
                self.keyboard_ui.write(e.EV_KEY, e.KEY_LEFTSHIFT, 1)
                self.keyboard_ui.syn()
            
            # Map character to key code
            key_code = getattr(e, f'KEY_{char.upper()}', None)
            if key_code is not None:
                self._keypress(key_code)
            elif char == " ":
                self._keypress(e.KEY_SPACE)
            elif char == "\n":
                self._keypress(e.KEY_ENTER)
            elif char == "\t":
                self._keypress(e.KEY_TAB)
            elif char == ".":
                self._keypress(e.KEY_DOT)
            # TODO: Add more mappings
            else:
                print(f"WARNING: Unhandled character: {char}")
            
            if char.isupper():
                # Release shift
                self.keyboard_ui.write(e.EV_KEY, e.KEY_LEFTSHIFT, 0)
                self.keyboard_ui.syn()
            
            time.sleep(0.05)  # Small delay between keystrokes

    def scroll(self, x: int, y: int, scroll_x: int, scroll_y: int) -> None:
        """Scroll at the specified position.
        
        Args:
            x: X coordinate to scroll at
            y: Y coordinate to scroll at
            scroll_x: Horizontal scroll amount (positive for right, negative for left)
            scroll_y: Vertical scroll amount (positive for down, negative for up)
        """
        # Move to the position first
        self.move_to(x, y)
        
        # Perform horizontal scroll if needed
        if scroll_x != 0:
            self.mouse_ui.write(e.EV_REL, e.REL_HWHEEL, -scroll_x)  # Invert horizontal scroll
            self.mouse_ui.syn()
            time.sleep(0.05)  # Small delay between scroll events
        
        # Perform vertical scroll if needed
        if scroll_y != 0:
            self.mouse_ui.write(e.EV_REL, e.REL_WHEEL, -scroll_y)  # Invert vertical scroll
            self.mouse_ui.syn()
            time.sleep(0.05)  # Small delay between scroll events
        
        # Small delay to ensure scroll is registered
        time.sleep(0.1)

    def wait(self, ms: int = 1000) -> None:
        time.sleep(ms / 1000)

    def screenshot(self) -> str:
        """Take a screenshot and return it as a base64 string."""
        screenshot = ImageGrab.grab()
        
        # Convert to bytes
        img_byte_arr = io.BytesIO()
        screenshot.save(img_byte_arr, format='PNG')
        img_byte_arr = img_byte_arr.getvalue()
        
        # Convert to base64
        base64_data = base64.b64encode(img_byte_arr).decode('utf-8')
        return base64_data


class WindowMPXEnvironment(MPXEnvironment):
    def __init__(self, window_id):
        super().__init__()
        self.window_id = window_id
        self.window_info = get_window_info(window_id)
        # Override width and height with window dimensions
        self.width = self.window_info['width']
        self.height = self.window_info['height']
        self._was_always_on_top = False

    def __enter__(self):
        # Store current window state
        d = display.Display()
        window = d.create_resource_object('window', self.window_id)
        states = window.get_full_property(d.intern_atom('_NET_WM_STATE'), Xatom.ATOM)
        self._was_always_on_top = states and d.intern_atom('_NET_WM_STATE_ABOVE') in states.value
        
        # Set window to always on top
        set_window_always_on_top(self.window_id, True)
        
        # Call parent's __enter__
        return super().__enter__()

    def __exit__(self, exc_type, exc_value, traceback):
        # Restore original window state
        if not self._was_always_on_top:
            set_window_always_on_top(self.window_id, False)
        
        # Call parent's __exit__
        super().__exit__(exc_type, exc_value, traceback)

    def move_to(self, x, y):
        # Convert window-relative coordinates to absolute screen coordinates
        abs_x = x + self.window_info['x']
        abs_y = y + self.window_info['y']
        
        super().move_to(abs_x, abs_y)
        print(f"Moved to {x}, {y} (window-relative) -> {abs_x}, {abs_y} (screen)")

    def screenshot(self) -> str:
        """Take a screenshot of the window and return it as a base64 string."""
        # Get window geometry
        geometry = get_window_info(self.window_id)
        
        # Take screenshot of the entire screen
        screenshot = ImageGrab.grab()
        
        # Crop to window region
        window_screenshot = screenshot.crop((
            geometry["x"],
            geometry["y"],
            geometry["x"] + geometry["width"],
            geometry["y"] + geometry["height"]
        ))
        
        # Convert to bytes
        img_byte_arr = io.BytesIO()
        window_screenshot.save(img_byte_arr, format='PNG')
        img_byte_arr = img_byte_arr.getvalue()
        
        # Convert to base64
        base64_data = base64.b64encode(img_byte_arr).decode('utf-8')
        return base64_data


def main():
    window = select_window()
    
    print(f"\nSelected window: {window['name']}")
    print(f"Window ID: {window['id']}")
    print(f"Size: {window['width']}x{window['height']}")
    
    # Create environment for the selected window
    with WindowMPXEnvironment(window['id']) as env:
        positions = [(0, 0), (env.width, 0), (0, env.height), (env.width, env.height)]
        for _ in range(10):  # Loop 10 times
            for x, y in positions:
                env.move_to(x, y)
                time.sleep(1)


if __name__ == "__main__":
    main()