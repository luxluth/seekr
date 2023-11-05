import os
from requests import get
import tomli

"""
# Example plugin
name = "app-launcher"
description = "Launches applications"
cmd = "@script:app-launcher"
run_on_any_query = true
priority = 3
dev = false
"""

PLUGINS_DIR = f"{os.getenv('HOME')}/.config/fsearch/plugins"
SCRIPTS_DIR = f"{os.getenv('HOME')}/.config/fsearch/scripts"


def get_plugin(plugin_name):
    with open(f"{PLUGINS_DIR}/{plugin_name}", "rb") as f:
        return tomli.load(f)

def list_plugins() -> list[dict]:
    plugins = []
    for plugin in os.listdir(PLUGINS_DIR):
        if plugin.endswith(".toml"):
            plug = get_plugin(plugin)
            plugins.append({
                "name": plug["name"],
                "description": plug["description"],
                "cmd": plug["cmd"],
                "run_on_any_query": plug["run_on_any_query"],
                "priority": plug["priority"],
                "dev": plug["dev"],
            })
    return plugins
    