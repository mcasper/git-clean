# git-clean

[![Build Status](https://travis-ci.org/mcasper/git-clean.svg?branch=master)](https://travis-ci.org/mcasper/git-clean)

# The Problem

If you work on one project for a long time, you're bound to amass a good number
of branches. Deleting these branches locally whenever you're done with them
gets annoying, and can cost you a lot of time in branch grooming, or trying to
remember 'that command' to delete all merged branches locally.

`git-clean` looks to remedy that. By running `git-clean`, you'll delete all
your merged branches quickly and easily.

# Other implementations

There are a couple other tools out there like this, but they all fall short for
me in some way.

https://github.com/arc90/git-sweep

This tool works great for smaller projects, but if you work on a large project
with tens or hundreds of thousands of commits, and thousands of active
branches, it stalls out. I've tried several times to get it to work on these
larger projects, but I've never been able to. It also has troubles deleting
branches locally if they've already been deleted in the remote.

https://github.com/mloughran/git-clean

This tool takes a slightly different approach, it will show you each branch
sequentially and let you decide what to do with it. This might work great for
some people, but I usually end up cleaning out my branches when the output of
`git branch` becomes unmanagable, so I would rather batch delete all my merged
branches in one go.

https://github.com/dstnbrkr/git-trim

This tool does something reminiscent of interactive rebasing, it will display
_all_ of your branches in your text editor, let you choose which ones you want
to delete, and deletes them upon saving. My problems with this are: It's a
manual process - and, - It doesn't only display merged branches, meaning that
you could delete branches that have valuable work on it.

# Advantages to this project

- Fast

This project is written in Rust, which is [really stinkin
fast](http://benchmarksgame.alioth.debian.org/u64q/rust.html). It takes about
1.8 seconds to delete 100+ branches, and most of that is network time.
`./target/release/git-clean  0.07s user 0.08s system 8% cpu 1.837 total`

- Batch operations

It deletes your branches in bulk, no stepping through branches or selecting
what branches you want gone. It assumes you want to delete all branches that
are even with your base branch.

- Deletes local and remote

It deletes both local and remote branches, and handles the errors if the remote
is already deleted.

- Only presents merged branches

There's no possibility of deleting branches with valuable work on them, as it
only deletes branches that are even with the base branch you specify (defaults
to main).

- Handles branches squashed by Github

Github recently introduced the ability to squash your merges from the Github
UI, which is a really handy tool to avoid manually rebasing all the time.
`git-clean` knows how to recognize branches that have been squashed by Github,
and will make sure they get cleaned out of your local repo.

# Assumptions

This tool assumes (but will also check) that your `git` is properly configured
to push and pull from the current repository. `git-clean` should be run from
the directory that holds the `.git` directory you care about.

This tool will run the `git` commands `branch`, `rev-parse`, `remote`, `pull`,
and `push` on your system. `git push` will only ever be run as `git push
<remote> --delete <branch>`, when deleting remote branches for you. If that
isn't acceptable, use the `-l` flag to only delete branches locally.

# Installation

If you're a Rust developer, you can install using Cargo:

```shell
cargo install git-clean
```

This was developed on Rust 1.14.0 stable, so if you're having issues with the
compile/install step, make sure your Rust version is >= 1.14.0 stable.

Be sure to add the installation path to your PATH variable. For me, it's
downloaded to:

```
/Users/mattcasper/.multirust/toolchains/stable/cargo/bin/git-clean
```

If you're not a Rust developer, or just prefer another way, there's also
a homebrew formula:

```shell
brew tap mcasper/formulae
brew install git-clean
```

Verify that it works!:

```shell
$ git-clean -h
USAGE:
    git-clean [FLAGS] [OPTIONS]

FLAGS:
    -d, --delete-unpushed-branches    Delete any local branch that is not present on the remote. Use this to speed up
                                      the checks if such branches should always be considered as merged
    -h, --help                        Prints help information
    -l, --locals                      Only delete local branches
    -r, --remotes                     Only delete remote branches
    -s, --squashes                    Check for squashes by finding branches incompatible with main
    -V, --version                     Prints version information
    -y, --yes                         Skip the check for deleting branches

OPTIONS:
    -b, --branch <branch>       Changes the base for merged branches (default is main)
    -i, --ignore <ignore>...    Ignore given branch (repeat option for multiple branches)
    -R, --remote <remote>       Changes the git remote used (default is origin)
```

# Updating

If you're updating from an older version of git-clean, and using Cargo to
install, just run the install command with `--force`:

```shell
cargo install git-clean --force
```

# Use

## git-clean

Lists all the branches to be deleted, and prompts you to confirm:

```shell
$ git-clean
The following branches will be deleted locally and remotely:
branch1
branch2
branch3
Continue? (Y/n)
```

If accepted, it will delete the listed branches both locally and remotely:

```shell
Continue? (Y/n) y

Remote:
 - [deleted]         branch1
 branch2 was already deleted in the remote.

 Local:
 Deleted branch branch1 (was 3a9ea97).
 Deleted branch branch2 (was 3a9ea97).
 Deleted branch branch3 (was 3a9ea97).
```

Branches that are already deleted in the remote are filtered out from the
output.

It also offers several options for tweaking what branches get deleted, where.

- `-l` and `-r` toggle deleting branches only locally or only remotely
- `-R` changes the git remote that remote branches are deleted in
- `-b` changes the base branch for finding merged branches to delete

And other miscellaneous options:

- `-y` overrides the delete branches check. Nice for automating workflows where
  you don't want to be prompted.

# Contributions

PRs and issues welcome!
