#import "/packages.typ": *

// applies the template
// the show rule essentially passes the entire document into the `notebook` function. 
#show: notebook.with(
  team-name: "53E", // TODO: put your team name here
  season: "High Stakes",
  year: "2024-2025",
  theme: radial-theme, // TODO: change the theme to one you like
)

#text(font: ("Calibri","Carlito"))[BLAH BLAH BLAH DDDDDDDDDD]

#include "./frontmatter.typ"

#lorem(100)

#include "./entries/entries.typ"

#include "./appendix.typ"
