#import "./packages.typ": notebookinator
#import notebookinator: *
#import themes.radial.components

#glossary.add-term(
  "Example word",
)[
  This is an example word which will appear in the glossary.
]

#create-appendix-entry(title: "Glossary")[
  #components.glossary()
]
