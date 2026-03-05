# saran

Lightweight tool for building CLI wrappers in Python using `subprocess`, [`click`](https://github.com/pallets/click) and [`yaml-reference`](https://github.com/dsillman2000/yaml-reference)

## Elevator pitch

Write a "walled garden" CLI wrapper around an existing CLI tool to support LLM agents freely using a CLI with limited functionality without direct user approval.

For instance, a specialized CLI wrapper around `gh` can be written which supports only reading issues and pull requests, but not creating them. These CLI wrappers for external services can be allowlisted by agents like GH Copilot, Claude Code or Devin to safely allow them to interact with the outside world in a way which doesn't give them freedom to perform destructive or undesired actions. The goal of this project is to make the design of these simple CLI wrappers as easy and intuitive as possible.

## Quick start

This example quickstart demonstrates a `saran` wrapper around `gh` designed for PR review. It is restricted such that it can only perform read-only operations on a specific pull request (#123) in a specific repository (octocat/hello-world).

```yaml
#!/usr/bin/env saran
name: gh-pr-review
description: |
  A CLI wrapper around `gh pr` commands to review the pull request with ID = 123 in the repository octocat/hello-world.
version: 1.0.0
commands:
  - name: view
    description: Display the title, body, and other information about the pull request.
    actions:
      - gh: [pr, view, -R, octocat/hello-world, 123]
```

Running this file with `saran` will act as a CLI shim which has only the "view" command defined, which invokes the underlying `gh pr view` command with the specific repo and pull request ID. If this were updated to include other contextual cues such as `gh pr diff` and `gh pr checks`, it would allow an LLM agent to use this CLI wrapper to view details about the pull request, but it without being able to perform any other actions like creating a new pull request or commenting on it.

Todo:

- [ ] Formalize templating system.
- [ ] More case studies ("integration testing," sort of).
- [ ] Unit tests
- [ ] Publish to PyPI
- [ ] Robustness test suite for prompt engineering (?)
