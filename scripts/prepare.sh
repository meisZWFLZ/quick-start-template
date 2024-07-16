#!/usr/bin/env bash
# This script is run inside the container whenever new content is available 
# in the source tree during the creation process.
# See updateContentCommand: https://containers.dev/implementors/json_reference/#lifecycle-scripts

# sets packages.typ to the installed version of notebookinator
sed -Ei packages.typ \
  -e "s%@local/notebookinator:[0-9]+\.[0-9]+\.[0-9]+%@local/notebookinator:$(
    ls ~/.local/share/typst/packages/local/notebookinator/ | head
  )%"
