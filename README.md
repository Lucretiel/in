# in

`in` is a trivial utility that runs a command in a specific directory. It is for the working directory what `env` is for environment variables. The usage is simple:

```
in /path/to/directory <command>...
```

Unless `-n`/`--no-pwd` is given, the `PWD` environment variable will be set to the target directory before the command is executed.
