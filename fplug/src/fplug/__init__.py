# SPDX-FileCopyrightText: 2023-present luxluth <delphin.blehoussi93@gmail.com>
#
# SPDX-License-Identifier: MIT
import requests
from about import __version__
from rich import print as rprint, print_json
import typer
from utils import list_plugins



app = typer.Typer(add_completion=True, name="fplug", help="fsearch plugin manager")



@app.command(help="Get fplug version")
def version():
    print(f"fplug v{__version__}")

@app.command(help="List installed plugins")
def list():
    plugins = list_plugins()
    rprint(plugins)

if __name__ == "__main__":
    app()