# Shrimp - Shellzao's Rust Implementation

`shrimp` ain't much, but it's a honest shell - written in Rust with (up until now) no additional dependencies, and with sufficient features to be human-usable and user-friendly.

## Installing
### Via Git, for Rusteceans
Currently the only way, clone the repo and use `cargo`

## Why 

### Use it?
Currently, you probably shouldn't - it's very barebones

### Make it?
Because in college I wrote a shell in C, the epynomous Shellzao, which wasn't very good in any aspect and I barely understood what I was doing - that inner demon needed exorcising.

## 

## Testing
Many tests doing redirections access a set of test files - it's probably a sub-optimal way to do it, but you need to run them single-thread and one module at a time or they will fail, will fix that eventually
`cargo test -- --test-threads=1`

## Built-in Commands

* [x] `cd <path>` makes the directory 'path' the current directory
* [x] `exit` terminates foosh
* [x] `quit` same as exit
* [ ] `fg [job-id]` sends job identified by jobid to foreground. If jobid is not specified, defaults to job which sate has been most recently modified.
* [ ] `bg [job-id]` sends job identified by jobid to background. If jobid is not specified, defaults to job which sate has been most recently modified.
* [ ] `jobs` output a list of currently active jobs  If a built-in command conflicts with the name of an external program, the built in command prevails --- unless the program path is explicitly given.
* [ ] `echo`
* [ ] `let` to set a new environment variable with a value.  


## Basic features
- [x] Command execution: built-ins and external
- [x] Pipelining
- [ ] `&&` and `||` logic 
- [ ] IO Redirection
  - [x] Files
  - [ ] Network
- [ ] Background Execution / Job management
- [ ] History
- [ ] Expansion

## More features
- [ ] Prompt customization
- [ ] Profiles / configs w/ variables
- [ ] Basic Scripting

## Advanced features
No guarantee of implementing
- [ ] Scripting compatibility w/ bash
- [ ] Autocompletion
- [ ] Parsing (Lexer -> Parser)

### Lexer
* The Lexical analyzer separates input into tokens.
* The lexer will read the input character by character and it will try to match the input with each token
* Regular expressions describe each token