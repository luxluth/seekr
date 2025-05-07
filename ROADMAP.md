# ROADMAP

## planed features

- [ ] extension system
- [ ] extensions repository
- [ ] rink ?

### extensions

1. Goals

- Allow users to create add-ons for _seekr_
- The plugin system should be as simple as possible.
- Extensions can be configured by the user

2. And Ways to achieve them

- Lua (easy, simple to integrate to rust)
- Add some level of GTK bindings for the lua interface. And useful functions.
- Extensions have a gtk box for themselves.
- Extensions run conditions: ANYQUERY, COMMAND (`/{extension} {query}`), CONTAINS

### extensions repository

_A place where to find or discover add-ons_

### rink ?

In addition to the calculator, a built-in tool
can be [rink](https://github.com/tiffany352/rink-rs/). Rink is a unit-aware calculator
that allow to make queries like `3 feet to meters` -> `0.9144 meter (length)`
