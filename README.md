git-clean
===========
# The Problem
If you work on one project for a long time, you're bound to amass a lot of
branches.  Deleting the branches locally gets annoying, and can cost you a lot
of time in branch grooming, or trying to remember 'that command' to delete all
merged branches locally.

`git-clean` looks to remedy that. By running `git-clean`, you'll delete all
your merged branches quickly and easily.

# Other implementations
There are a couple other tools out there like this, but they all fall short for
me in some way.

https://github.com/arc90/git-sweep

This tool works great for smaller projects, but if you work on a large project
with tens or hundreds of thousands on commits, it stalls out. I've tried
several times to get it to work on these larger projects, but I've never been
able to. It also seems to blow up if the branch is already deleted remotely.

https://github.com/mloughran/git-clean

This tool takes a slightly different approach, it will show you each branch and
let you decide what to do with it. This might work great for some people, but I
usually end up cleaning out my branches when the output of `git branch` becomes
unmanagable, so I would rather batch delete all my merged branches than walk
through them one by one.

https://github.com/dstnbrkr/git-trim

Only does local projects, and requires editing. Does not focus on branches
merged into master.

# Advantages to this project
- Fast

This project is written in Rust, which is [really stinkin fast](). It takes
about Nms to delete 100+ branches.

- Batch operations

It deletes your branches in bulk

- Deletes local and remote

It deletes both local and remote branches, and handles if the remote is already
deleted

# Assumptions
This tool assumes that you have `git` installed, and is on your path. If you
don't have it installed, I'm confused as to why you've read this far, but go
[here]() to download it.

It also assumes that your `git` is properly configured to push and pull from
the current repository. `git-clean` should be run from the directory that
holds the `.git` directory you care about.

This tool will run the commands `git branch`, `git rev-parse`, and `git push`
on your system.  `git push will only ever be run as `git push origin --delete
'remote_branch'`, when deleting remote branches for you. If that isn't
acceptable, use the `-l` flag to only delete branches locally.

# Use
## git-clean
Lists all the branches to be deleted, and prompts you to confirm:
```
The following branches will be deleted locally and remotely:
branch1
branch2
branch3
Continue? (yN)
```

If accepted, it will delete the listed branches both locally and remotely:
```
RESULT HERE
```

Deletes all merged branches, both local and remote.
```
Deleting 20 branches from <repo>
Progress: [#######     ] 9/20
Successfully deleted these branches:
l_branch
r_branch
common
```

It also offers the `--local` and `--remote` flags. These do exactly what you
think, they'll only delete local branches or remote branches respectively.

# Contributions
PRs welcome!