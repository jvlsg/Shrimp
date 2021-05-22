# Pseudo BNF for Shrimp's grammar
```
<LIST> ::= <SUBLIST ;> | <SUBLIST &> | <SUBLIST>
<SUBLIST> => <PIPELINE && SUBLIST` | `PIPELINE || SUBLIST` | `PIPELINE`

<PIPELINE> ::= <CMD> <STD_PIPE> <PIPELINE> | <CMD> <PIPE> <PIPELINE>  | <CMD>
<STD_PIPE> ::= "|"
<ERR_PIPE> ::= "|"
<CMD> ::= command\n | command;
```

# Command
From ZSH's grammar definition

> A simple command is a sequence of optional parameter assignments followed by blank-separated words, with optional redirections interspersed.
> Every command ends with either a newline (by pressing the return key) or a semicolon ;

## Redirections
* `<`: Read File as input. If file does not exists, it fails
* `>`: Write Output to a new File, or Overwrite file if existing
* `>>` Write new, or Append if existing, File as Output
* `&>` or `>&` Redirects Stdout and Stderr to the file
* `&>>` or `>>&` Redirects Stdout and Stderr to the file, Appending it.