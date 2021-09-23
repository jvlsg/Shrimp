# Shrimp
Shellzao Rust Implementation

## Test

`cargo test -- --test-threads=1`


## Design Notes

### General
* Basic Unit of execution is the Pipeline.
* The Pipeline is composed by Steps (other shells sometimes use "Simple Command" as terminology).
* Steps are either a built-in functions, or external programs
* Redirections (and possibly pipes?) require whitespace as delimitators

### Steps
* Implemented as an enum instead of trait
  * Known and limited number of variants (either a Built-in, or external command)
  * No need for extensibility (one of the main uses for Traits and Trait Objects)
  * More performant than trait objects due to lack of dynamic dispatching (at least in theory, not sure how big the impact would be in such a small case)
* Pipes connect the output of one Step with the input of the next by passing byte streams (`Vec<u8>`)
  * Flexible, we can use `Read`ers and `Write`ers

## Sources
- https://gitlab.com/monaco/posixeg/-/blob/master/exercises/shell/foosh.txt
- https://doc.rust-lang.org/std/process/index.html
- https://www.joshmcguigan.com/blog/build-your-own-shell-rust/
- https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features
- https://github.com/psinghal20/rush
- https://hyperpolyglot.org/unix-shells
- http://zsh.sourceforge.net/Doc/Release/Shell-Grammar.html
- https://github.com/Swoorup/mysh
- https://adriann.github.io/rust_parser.html

## Built-in Commands

* `cd <path>` makes the directory 'path' the current directory
* `exit` terminates foosh
* `quit` same as exit
* `fg [job-id]` sends job identified by jobid to foreground. If jobid is not specified, defaults to job which sate has been most recently modified.
* `bg [job-id]` sends job identified by jobid to background. If jobid is not specified, defaults to job which sate has been most recently modified.
* `jobs` output a list of currently active jobs  If a built-in command conflicts with the name of an external program, the built in command prevails --- unless the program path is explicitly given.
* `echo`


## Basic features
- [x] Command execution: built-ins and external
- [x] Pipelining
- [ ] IO Redirection
  - [x] Files
  - [ ] Network
- [ ] Background Execution / Job management
- [ ] Basic Scripting

## More features
- [ ] Prompt customization
- [ ] Profiles / configs w/ variables
- [ ] History
- [ ] Expansion
- [ ] Proper parsing

## Advanced features
No guarantee of implementing
- [ ] Scripting compatibility w/ bash
- [ ] Autocompletion
- [ ] Parsing (Lexer -> Parser)

### Lexer
* The Lexical analyzer separates input into tokens.
* The lexer will read the input character by character and it will try to match the input with each token
* Regular expressions describe each token