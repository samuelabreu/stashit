# Stash It

git stash command for non-repo files.

## Description

git stash is able to store files on a temporary area, i believe is a helpful
feature to have for random files on you filesystem instead of creating .bkp or
similar files.

git stash works keeping only diffs from previous file on repo, stash it keeps
full copies of files on a backup directory.

# Usage

Stash three files:

```
stashit list.md of.rs files.sh 
```

To avoid removing files use --keep or -k:

```
stashit -k list.md of.rs files.sh
```

List all files stashed:

```
stashit -l
```

Pop (restore and remove from stash) last stash:

```
stashit -p
```

Remove stash number N:

```
stashit -r N
```

# Build

stash it is made with rust, and can be built using cargo:

```
cargo build
```

or run directly

```
cargo run
```

# Configuration

It stores files stashed by default in path:
```
~/.local/share/stashit/
```

It can be changed using a config file:

~/.config/stashit/stashit.toml
```
path = '~/.custom_path/'
```
