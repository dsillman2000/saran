---
name: gh-pr-review
description: A CLI tool for reviewing pull requests using GitHub's `gh` command-line interface
---
# gh-pr-review Skill

A CLI tool for reviewing pull requests using GitHub's `gh` command-line interface.

You have the `gh-pr-review` CLI tool installed on your PATH. This tool provides commands to view PR details, show code diffs, check CI/CD status, and check out PR branches locally.

## Usage

```
gh-pr-review <command> [options]
```

## Commands

- **view** - Display PR title, description, author, and other metadata
- **diff** - Show all code changes in the pull request
  - Optional flag: `--patch` - Show changes in patch format
- **checks** - Display CI/CD status and checks for the PR
  - Optional flag: `--json` - Output check results in JSON format with specific fields (comma-separated). Fields may be any of: [bucket, completedAt, description, event, link, name, startedAt, state, workflow]
- **checkout** - Check out the PR branch locally in git

## Examples

```
gh-pr-review view
gh-pr-review diff
gh-pr-review diff --patch
gh-pr-review checks
gh-pr-review checks --json name,bucket,state
gh-pr-review checkout
```
