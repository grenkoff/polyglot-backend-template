---
description: Show the registry of design patterns before writing a new object
argument-hint: "[what the new object should do]"
---

Run `make patterns` and read `docs/patterns.md`.

Then answer for the current task ($ARGUMENTS):

1. Which objects in the registry already solve a similar problem, with their files.
2. Whether the task can reuse one of them or wrap it in a decorator instead of adding a new object.
3. If a new pattern object is still needed, why it earns its place under "Design patterns" in `AGENTS.md` (rule of three, no speculative patterns, the lighter form first).

If `make patterns` changed `docs/patterns.md`, say so: the registry is committed together with the code that changed it.
