import subprocess
from evdev import UInput, ecodes as e, AbsInfo
import time
import re
import base64
from PIL import Image, ImageGrab
import io


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


class MPXEnvironment:
    def __init__(self):
        self.width, self.height = get_screen_resolution()

    def __enter__(self):
        # Create a virtual mouse device
        mouse_capabilities = {
            e.EV_ABS: [
                (e.ABS_X, AbsInfo(0, 0, self.width - 1, 0, 0, 0)),
                (e.ABS_Y, AbsInfo(0, 0, self.height - 1, 0, 0, 0)),
            ],
            e.EV_KEY: [e.BTN_LEFT, e.BTN_RIGHT],
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
        self.mouse_ui.close()
        self.keyboard_ui.close()

        # Remove the MPX master device
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

    def press_key(self, key_code):
        """Press and release a key."""
        self.keyboard_ui.write(e.EV_KEY, key_code, 1)
        self.keyboard_ui.syn()
        time.sleep(0.05)
        self.keyboard_ui.write(e.EV_KEY, key_code, 0)
        self.keyboard_ui.syn()

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
                self.press_key(key_code)
            
            if char.isupper():
                # Release shift
                self.keyboard_ui.write(e.EV_KEY, e.KEY_LEFTSHIFT, 0)
                self.keyboard_ui.syn()
            
            time.sleep(0.05)  # Small delay between keystrokes

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


def main():
    with MPXEnvironment() as env:
        positions = [(100, 100), (env.width // 2, env.height // 2), (env.width - 200, env.height - 200), (100, 100)]
        time.sleep(5)
        for x, y in positions:
            env.move_to(x, y)
            time.sleep(1)


if __name__ == "__main__":
    main()