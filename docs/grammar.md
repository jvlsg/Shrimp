# Pseudo BNF for Shrimp's grammar
```
<LIST> ::= <SUBLIST ;> | <SUBLIST &> | <SUBLIST>
<SUBLIST> => <PIPELINE && SUBLIST` | `PIPELINE || SUBLIST` | `PIPELINE`

<PIPELINE> ::= <STEP> <STD_PIPE> <PIPELINE> | <STEP> <ERR_PIPE> <PIPELINE>  | <STEP>
<STD_PIPE> ::= "|"
<ERR_PIPE> ::= "|&"
<STEP> ::= step\n | step;
```


## Redirections
* `<` Read File as input. If file does not exists, it fails
* `>` Write Output to a new File, or Overwrite file if existing
* `>>` Write new, or Append if existing, File as Output
* `&>` Redirects Stdout and Stderr to the file
* `&>>` Redirects Stdout and Stderr to the file, Appending it.

## Piping
* `|` Standard Pipe, standard output of one command is connected to the next command's standard input
* `|&` Connect the first command's standard error, in addition to its standard output, to the next command input
