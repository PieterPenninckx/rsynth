#Contributing
Contributions and suggestions are welcome!

## Opening and voting for issues

If there is a feature you would like to see, feel free to open an issue or "vote" for an issue by
adding the "thumbs up" emoji.

## Reviewing pull requests

Two pair of eyes see more than just one. Have a look at 
[this issue](https://github.com/PieterPenninckx/rsynth/issues/74) if you want to help by reviewing
code.

## Updating documentation

Everybody loves good documentation. Contributing to the doc comments is a way to contribute that does not require that many
skills, but which has a big impact. For practical aspects, see "Contributing code" below.

## Contributing code

Code contributions are certainly welcome as well!

In order to avoid pull requests from being broken by other changes, please open an issue or
have an issue assigned to you before you start working on something. 
In this way, we can coordinate development.
Issues labeled with "good first issue" should not conflict too much with other changes
that are in flight, but better check before you start working on one.

Don't worry if you only have a partial solution. You can still open a pull request for that. 
You can definitely split the solution for an issue into different pull requests. 

I tend to squash all commits, which means that all your intermediate commits are combined into
one commit. This has the advantage that you don't need to worry about what is in these intermediate
commits. On the other hand, some people want to have more activity on their GitHub timeline. If
you don't want me to squash the commits, let me know when you open the pull request.

Pull requests should be opened against the `master` branch.

### Code formatting
Please use `cargo fmt` to format your code before opening a pull request.

_Tip_: you can auto-format your code on save in your IDE:
* IntelliJ: `File > Settings > Languages & Frameworks > Rust > Rustfmt > Run rustfmt on save`
* [Visual Studio Code with `rls-vscode`](https://github.com/rust-lang/rls-vscode#format-on-save)

## Testing

In order to run all tests, run the following:
```bash
cargo test --features all
```

If you have trouble running this locally because you do not have jack-related libraries installed,
no worries: you can still open a pull request; this will automatically trigger a CI build that runs
all tests for you.