<img src="launcher/images/uiact_bw.svg" alt="UI Act" width="200" />

**UI Act** is a Computer Use/GUI agent software that works alongside you on your Linux desktop. Just press `CTRL + Space`, type your prompt and the agent will kick off — **_using its own mouse and keyboard_** . Have a look:

[Demo](https://github.com/user-attachments/assets/78a4ddf8-f0f2-4c00-85a1-37a2e4bcb55f)

This way, you can still continue working as the agent doesn't hog your mouse. Yet, it allows for a seamless agent handoff as it's working directly in your desktop environment! It works by using [Multi-Pointer X](https://en.wikipedia.org/wiki/Multi-Pointer_X) (MPX), a feature for having multiple mouse pointers in the X windowing system. UI Act is completely free and open source (Apache License 2.0), and you stay in control of your data by providing your own API key. Currently UI Act supports Anthropic as Computer Use backend.

**Note:** Looking for UI Act _the model_ (an early computer use model)? It has been moved [here](https://github.com/TobiasNorlund/UI-Act-model)

## ⚡️ Quick start

UI Act currently only runs on Ubuntu Desktop 24.04 and later, and is distributed as a debian package. Run this in a terminal to set it up:

```bash
# Download and install deb package file
wget https://github.com/TobiasNorlund/ui-act/releases/download/v0.1.0/ui-act_0.1.0_amd64.deb
sudo apt install ./ui-act_0.1.0_amd64.deb

# Add your user to the "input" group. This is necessary for your user to be able to create and use UInput devices
sudo usermod -aG input $USER
```

Now log out and log back in again for the changes to take effect. Then run:

```bash
# Enable UI Act GNOME extension
gnome-extensions enable ui-act@tobiasnorlund.github.com

# Open settings and add your Anthropic API key in the settings window
gnome-extensions prefs ui-act@tobiasnorlund.github.com
```

You are now set up to use UI Act! Try press `CTRL + Space` and kick off a prompt!

**Note:** You will need to run your session using X11, not Wayland. Check what your're running with `echo $XDG_SESSION_TYPE`. To switch, log out and on the login screen, click on your username to activate the password field. Click the gear icon in the bottom right corner and select "Ubuntu on Xorg" before logging in.


## 🛠️ How it works

The package installs a CLI tool `ui-act` that runs the agent, and a GNOME extension to quickly launch agents in GNOME.

### CLI

The `ui-act` command support running a GUI agent across the full desktop, or in a "single window" mode. In single window mode, the agent only gets screenshots and can only act in this window. To ensure the window is not obstructed, it is set to "Always on top" for as long as the agent runs.

```
ui-act [--window <window_id>] [--no-telemetry] [--help] <prompt>
```

- `--window <window_id>` - (optional) If provided an X window id (obtainable via e.g. `xwininfo`), run in "single window" mode.
- `--no-telemetry` - Anonymous usage statistics is sent for improving UI Act, but you can opt-out by providing this flag. No user data (prompts, screenshots or api keys) is sent. See for yourself [in the code](ui_act/src/telemetry.rs).
- `prompt` - A string like "In the open browser, go to Amazon and find me some Ray-Ban Meta Glasses"

When starting an agent, two things happen: 1) a new xinput master is created behind the scenes (check it with `watch xinput` while running the agent). 2) Virtual (UInput) mouse and keyboard devices are created and attached to the xinput master, through which the agent can act.

As the agent acts, its reasoning and steps are printed in the output, and if it needs clarifications or finishes, the user is prompted for additional input.

The agent can be interrupted by pressing `CTRL+C` in the terminal.

**Note:** As of now, the created xinput master device remains even after the agent exits (though the pointer is moved to the bottom right corner of the screen). This is due to some applications crashing upon removal of it. However, other applications (e.g. Chrome) have been noted to stop receiving keyboard input when the xinput master is kept. To fix this, simply remove the xinput master manually:

```bash
master_id=$(xinput list | grep "UI Act pointer" | grep -o 'id=[0-9]*' | cut -d= -f2)
xinput remove-master $master_id
```

### GNOME Extension

When enabled, the GNOME extension adds a clickable icon to the top panel and hot keys (`CTRL + Space`) for launching UI Act (in single-window mode only as of now).


## 🌟 Vision and Roadmap

AI is today dramatically boosting productivity for programmers, but its potential extends far beyond coding. Computer Use has immense potential to empower knowledge workers in other fields, but hinges on delivering an exceptional user experience and ensuring such agents are intelligent, safe, and reliable.

Much still remains, but here is a short list of what's being planned for UI Act (contributions are welcome!):

 - [ ] Add a lovable UI for monitoring and interacting with the agent
 - [ ] Support additional models (Open AI, self hosting etc.)
 - [ ] Allow for context engineering to improve agent reliability
 - [ ] Extend single-window mode to multi-window mode
 - [ ] Allow running agent in the background via Xephyr
 - [ ] Add guardrails
 - [ ] Got ideas? Awesome! Create an Issue and tag as _enhancement_


## 💬 Feedback

We'd love to hear your feedback. Please let us know about potential problems, bugs or suggestions for improvement your experience by creating in Issue and tag it accordingly.

## 🤝 Contributing

Contributions to UI Act are very welcome! The process is simple:

1. Start by creating an Issue for what you would like to do and express your interest in implementing it.
2. Fork the repo, add a branch for your changes and submit a Pull Request.

