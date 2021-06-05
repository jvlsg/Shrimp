# Shrimp
Shellzao Rust Implementation


## Design Notes
* Basic Unit of execution is the Pipeline.
* The Pipeline is composed by Steps (other shells sometimes use "Simple Command" as terminology).
* Steps are either a built-in functions, or external programs
* Redirections (and possibly pipes?) require whitespace as delimitators

## Sources
- https://gitlab.com/monaco/posixeg/-/blob/master/exercises/shell/foosh.txt
- https://doc.rust-lang.org/std/process/index.html
- https://www.joshmcguigan.com/blog/build-your-own-shell-rust/
- https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features
- https://github.com/psinghal20/rush
- https://hyperpolyglot.org/unix-shells
- http://zsh.sourceforge.net/Doc/Release/Shell-Grammar.html
- https://github.com/Swoorup/mysh

## Built-in Commands

* `cd <path>` makes the directory 'path' the current directory
* `exit` terminates foosh
* `quit` same as exit
* `fg [job-id]` sends job identified by jobid to foreground. If jobid is not specified, defaults to job which sate has been most recently modified.
* `bg [job-id]` sends job identified by jobid to background. If jobid is not specified, defaults to job which sate has been most recently modified.
* `jobs` output a list of currently active jobs  If a built-in command conflicts with the name of an external program, the built in command prevails --- unless the program path is explicitly given.
* `echo`


## Basic features
- [ ] Command execution: built-ins and external
- [ ] Pipelining
- [ ] IO Redirection
- [ ] Background Execution / Job management
- [ ] Basic Scripting

## More features
- [ ] Prompt customization
- [ ] Profiles / configs w/ variables
- [ ] History
- [ ] Expansion

## Advanced features
No guarantee of implementing
- [ ] Scripting compatibility w/ bash
- [ ] Autocompletion