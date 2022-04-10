# temple
Wizard creator of wizards.

Use case: I have a project from `X` framework that has a very specific structure for it to work. I want to quickly create new projects based on that project and be able to customize very basic things from it.

```
.
└── my_latex_template
   ├── bibliography.bib
   ├── justfile
   └── test.tex 
```

`temple` allows to define all replaces that should be done for the files inside the directory.
For example, my `my_latex_template/test.tex` looks something like the following:

```latex
...
\begin{center}
    {\huge TITLE}\\
    ...
\end{center}
...
```

Use `temple` to create a wizard that handles that automatic creation with user-defined values.

```
temple my_latex_template newtex project -r "TITLE...The title of the document" "LaTex Wizard"
```

Hence, `temple` produces a binary named `newtex` that packs the folder `my_latex_template` that, when unpacked, replaces all `TITLE` by a user given string as an argument. The default name of any project created by the `newtex` wizard is `"project"` and its description is `"LaTex Wizard"`.

```
newtex
LaTex Wizard

USAGE:
    newtex [OPTIONS] --title <TITLE> [PATH]

ARGS:
    <PATH>    Where to create the project [default: .]

OPTIONS:
    -h, --help             Print help information
    -n, --name <NAME>      Name of the project [default: project]
    -t, --title <TITLE>    The title of the document
```

And now I can quickly create latex projects with the structure I want with customization options.

```
newtex --name l_algebra --title "Linear algebra"
```

And title has been remplaced inside `l_algebra/test.tex`

```latex
...
\begin{center}
    {\huge Linear algebra}\\
    ...
\end{center}
...
```
