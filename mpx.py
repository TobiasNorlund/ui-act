import subprocess
from evdev import UInput, ecodes as e, AbsInfo
import time
import re


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
        capabilities = {
            e.EV_ABS: [
                (e.ABS_X, AbsInfo(0, 0, self.width - 1, 0, 0, 0)),
                (e.ABS_Y, AbsInfo(0, 0, self.height - 1, 0, 0, 0)),
            ],
            e.EV_KEY: [e.BTN_LEFT, e.BTN_RIGHT],
        }
        self.ui = UInput(capabilities, name="CoX Mouse Device")
        mouse_id = get_device_id("CoX Mouse Device")
        if mouse_id is None:
            raise RuntimeError("Could not find device ID for 'CoX Mouse Device'")

        # Create a new MPX master device
        result = subprocess.run(['xinput', 'create-master', 'CoX'])
        if result.returncode != 0:
            raise RuntimeError("Failed to create MPX master device")
        self.master_id = get_device_id("CoX pointer")
        
        # Attach the virtual mouse to the MPX master device
        result = subprocess.run(['xinput', 'reattach', str(mouse_id), str(self.master_id)])
        if result.returncode != 0:
            raise RuntimeError("Failed to attach virtual mouse to MPX master device")
        
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        # Remove the virtual mouse device
        self.ui.close()

        # Remove the MPX master device
        result = subprocess.run(['xinput', 'remove-master', str(self.master_id)])
        if result.returncode != 0:
            raise RuntimeError("Failed to remove MPX master device")

    def move_to(self, x, y):
        self.ui.write(e.EV_ABS, e.ABS_X, x)
        self.ui.write(e.EV_ABS, e.ABS_Y, y)
        self.ui.syn()
        print(f"Moved to {x}, {y}")

    def left_click(self):
        self.ui.write(e.EV_KEY, e.BTN_LEFT, 1)
        self.ui.syn()
        time.sleep(0.1)
        self.ui.write(e.EV_KEY, e.BTN_LEFT, 0)
        self.ui.syn()

    def right_click(self):
        self.ui.write(e.EV_KEY, e.BTN_RIGHT, 1)
        self.ui.syn()
        time.sleep(0.1)
        self.ui.write(e.EV_KEY, e.BTN_RIGHT, 0)
        self.ui.syn()


def main():
    with MPXEnvironment() as env:
        positions = [(100, 100), (env.width // 2, env.height // 2), (env.width - 200, env.height - 200), (100, 100)]
        
        for x, y in positions:
            env.move_to(x, y)
            time.sleep(1)


if __name__ == "__main__":
    main()