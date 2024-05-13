#import "/packages.typ": notebookinator
#import notebookinator: *
#import themes.radial: radial-theme, components

#show: notebook.with(
  team-name: "53E", // TODO: put your team name here
  season: "High Stakes",
  year: "2024-2025",
  theme: radial-theme,
)

#include "./frontmatter.typ"

#include "./entries/entries.typ"

#include "./appendix.typ"
