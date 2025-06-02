from mpx import MPXEnvironment
import os
import requests


def create_response(**kwargs):
    url = "https://api.openai.com/v1/responses"
    headers = {
        "Authorization": f"Bearer {os.getenv('OPENAI_API_KEY')}",
        "Content-Type": "application/json"
    }

    openai_org = os.getenv("OPENAI_ORG")
    if openai_org:
        headers["Openai-Organization"] = openai_org

    response = requests.post(url, headers=headers, json=kwargs)

    if response.status_code != 200:
        print(f"Error: {response.status_code} {response.text}")

    return response.json()


def main():
    with MPXEnvironment() as computer:
        """Run the CUA (Computer Use Assistant) loop"""

        tools = [
            {
                "type": "computer-preview",
                "display_width": computer.width,
                "display_height": computer.height,
                "environment": "linux",
            }
        ]

        print(tools)

        items = []
        while True:  # get user input forever
            user_input = input("> ")
            items.append({"role": "user", "content": user_input})

            while True:  # keep looping until we get a final response
                response = create_response(
                    model="computer-use-preview",
                    input=items,
                    tools=tools,
                    truncation="auto",
                )

                if "output" not in response:
                    print(response)
                    raise ValueError("No output from model")

                items += response["output"]

                for item in response["output"]:
                    items += handle_item(item, computer)

                if items[-1].get("role") == "assistant":
                    break


def handle_item(item, computer: MPXEnvironment):
    """Handle each item; may cause a computer action + screenshot."""

    if item["type"] == "message":  # print messages
        print(item["content"][0]["text"])

    if item["type"] == "computer_call":  # perform computer actions
        action = item["action"]
        action_type = action["type"]
        action_args = {k: v for k, v in action.items() if k != "type"}
        print(f"{action_type}({action_args})")

        # give our computer environment action to perform
        getattr(computer, action_type)(**action_args)

        screenshot_base64 = computer.screenshot()

        pending_checks = item.get("pending_safety_checks", [])
        for check in pending_checks:
            print("Safety check:", check["message"])

        # return value informs model of the latest screenshot
        call_output = {
            "type": "computer_call_output",
            "call_id": item["call_id"],
            "acknowledged_safety_checks": pending_checks,
            "output": {
                "type": "input_image",
                "image_url": f"data:image/png;base64,{screenshot_base64}",
            },
        }

        return [call_output]

    return []


if __name__ == "__main__":
    main()