# Design Notes

## General
* Basic Unit of execution is the Pipeline.
* The Pipeline is composed by Steps (other shells sometimes use "Simple Command" as terminology).
* Steps are either a built-in functions, or external programs
* Redirections (and possibly pipes?) require whitespace as delimitators

## Steps
* Implemented as an enum instead of trait
  * Known and limited number of variants (either a Built-in, or external command)
  * No need for extensibility (one of the main uses for Traits and Trait Objects)
  * More performant than trait objects due to lack of dynamic dispatching (at least in theory, not sure how big the impact would be in such a small case)
* Pipes connect the output of one Step with the input of the next by passing byte streams (`Vec<u8>`)
  * Flexible, we can use `Read`ers and `Write`ers

# Sources / Useful links
- https://gitlab.com/monaco/posixeg/-/blob/master/exercises/shell/foosh.txt
- https://doc.rust-lang.org/std/process/index.html
- https://www.joshmcguigan.com/blog/build-your-own-shell-rust/
- https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features
- https://github.com/psinghal20/rush
- https://hyperpolyglot.org/unix-shells
- http://zsh.sourceforge.net/Doc/Release/Shell-Grammar.html
- https://github.com/Swoorup/mysh
- https://adriann.github.io/rust_parser.html