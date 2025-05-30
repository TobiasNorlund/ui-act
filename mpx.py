import subprocess
from evdev import UInput, ecodes as e, AbsInfo
import time
import re


def get_screen_resolution():
    # Run xdpyinfo and parse dimensions line
    xdpyinfo = subprocess.run(['xdpyinfo'], capture_output=True, text=True)
    match = re.search(r'dimensions:\s+(\d+)x(\d+)', xdpyinfo.stdout)
    if match:
        width = int(match.group(1))
        height = int(match.group(2))
        return width, height
    else:
        raise RuntimeError("Could not detect screen resolution")

def main():
    width, height = get_screen_resolution()
    print(f"Detected screen resolution: {width}x{height}")

    capabilities = {
        e.EV_ABS: [
            (e.ABS_X, AbsInfo(0, 0, width - 1, 0, 0, 0)),
            (e.ABS_Y, AbsInfo(0, 0, height - 1, 0, 0, 0)),
        ],
        e.EV_KEY: [e.BTN_LEFT],
    }

    ui = UInput(capabilities, name="Virtual Absolute Mouse")

    positions = [(100, 100), (width // 2, height // 2), (width - 200, height - 200), (100, 100)]

    try:
        for x, y in positions:
            ui.write(e.EV_ABS, e.ABS_X, x)
            ui.write(e.EV_ABS, e.ABS_Y, y)
            ui.syn()
            print(f"Moved to {x}, {y}")
            time.sleep(1)
    except KeyboardInterrupt:
        pass
    finally:
        ui.close()
        print("Device closed")


if __name__ == "__main__":
    main()