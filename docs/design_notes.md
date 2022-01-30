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

## Expansion / Pre-processing

### Essentials

- [x] `$` env variables expansion

- [ ] `*` String Wildcard Pathname expansion. matches any (possibly empty) sequence of characters.
  * `D*` , `*.rs` , `/usr/*/share`

- [ ] `**` Recursive string Pathname expansion , matches the current directory and arbitrary subdirectories - Must be an entire path component
  - `a/**/b` is valid, but `a**/b` or `a/**b` is not
  - `***` or more is also invalid

- [ ] `?` Character Wildcard Pathname expansion
  * `ls image?.png` for `image1.png`, `image2.png`, etc...

- [ ] `[...]` Character Wildcard Pathname expansion
  * `ls /etc/[ab]*.conf`, etc...

- [x] `~` Expansion for user's home
  * When used at the beginning of a word, it expands into the name of the home directory of the named user, or if no user is named, the home directory of the current user:
  * ~~`~foo/` for user foo (?)~~ Nope, laziness



#### Quoting - Supress expansion
  * [x] `'` - **Supresses** ALL expansions
  * [x] `"` - If we place text inside double quotes, all the special characters used by the shell lose their special meaning and are treated as ordinary characters. The **exceptions** are “$”, “\” (backslash), and “`” (back- quote). This means that **word-splitting, pathname expansion, tilde expansion, and brace expansion are suppressed**, but parameter expansion, arithmetic expansion, and command substitution are still carried out
  * [x] `\` - It preserves the literal value of the next character that follows, with the exception of newline. If a \newline pair appears, and the backslash itself is not quoted, the \newline is treated as a line continuation (that is, it is removed from the input stream and effectively ignored). 
    * Ignore new line
    * \n 	newline 	Adding blank lines to text
    * \t 	tab 	Inserting horizontal tabs to text
    * \a 	alert 	Makes our terminal beep
    * \\ 	backslash 	Inserts a backslash
    * \f 	formfeed 	Sending this to our printer ejects the page

### Possibly
* [ ] `{A,B,C}` brace expansion
  * `,` = OR
  * `..` = From, to, inclusive
  * `echo Front-{A,B,C}-Back` => `Front-A-Back Front-B-Back Front-C-Back`
  * `echo Number_{1..5}` => `Number_1 Number_2 Number_3 Number_4 Number_5`
* [ ] Command Substitution? `echo $(ls)`

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
- https://linuxcommand.org/lc3_lts00802.php
- https://docs.rs/shellexpand/2.1.0/shellexpand/
- https://docs.rs/dirs/4.0.0/dirs/fn.home_dir.html
- https://docs.rs/glob/0.3.0/glob/