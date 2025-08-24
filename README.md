# UI Act

UI Act is a Linux desktop (aka computer using) agent that can perform tasks just like you would do them - by interacting live with your desktop GUI.

UI Act can work along side you using its own separate mouse and keyboard. It uses Multi-Pointer X (MPX), supported by the X windowing system, to interact with open windows on your desktop. This allows for a _true_ copilot experience where you and your agent can work side-by-side to get things done:

[TBA: Demo video]

## Install

```bash
# Download and install deb package file
sudo apt install <path to ui-act deb file>

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

You are now ready to use UI Act!


## 🚧 WIP

This project is under active development. If this sounds interesting and you would like to help out, please reach out to tobias@norlund.se
