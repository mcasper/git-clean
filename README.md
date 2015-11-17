git-clean
===========
# The Problem
If you work on one project for a long time, you're bound to amass a good number
of branches.  Deleting the branches locally whenever you're done with them gets
annoying, and can cost you a lot of time in branch grooming, or trying to
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
*all* of your branches in your text editor, let you choose which ones you want
to delete, and deletes them upon saving.  My problems with this are: It's a
manual process - and, - It doesn't only display merged branches, meaning that
you could delete branches that have valuable work on it.

# Advantages to this project
- Fast

This project is written in Rust, which is [really stinkin fast](). It takes
about 1.8 seconds to delete 100+ branches, and most of that is network time.
`./target/release/git-clean  0.07s user 0.08s system 8% cpu 1.837 total`

- Batch operations

It deletes your branches in bulk

- Deletes local and remote

It deletes both local and remote branches, and handles the errors if the remote
is already deleted

- Only presents merged branches

There's no possibly of deleting branches with valuable work on it, as it only
deletes branches that are even with the base branch you specify (default
master)

# Assumptions
This tool assumes that you have `git` installed, and is in your path. If you
don't have it installed, I'm confused as to why you've read this far, but go
[here]() to download it.

It also assumes that your `git` is properly configured to push and pull from
the current repository. `git-clean` should be run from the directory that
holds the `.git` directory you care about.

This tool will run the commands `git branch`, `git rev-parse`, `git remote` and
`git push` on your system.  `git push will only ever be run as `git push origin
--delete 'remote_branch'`, when deleting remote branches for you. If that isn't
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

It also offers serveral options for tweaking what branches get deleted, where.

* `-l` and `-r` toggle deleting branches only locally or only remotely
* `-R` changes the git remote that remote branches are deleted in
* `-b` changes the base branch for finding merged branches to delete

# Contributions
PRs and issues welcome!
