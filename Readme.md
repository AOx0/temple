# temple

Fast experimental project template renderer with easy setup and usage.

</br>

## Installation

    cargo install --git https://github.com/AOx0/temple

</br>

## Why temple?



### 1. It's fast.

It is fast, *very fast*. As shown in the benchmark results, where each tool rendered the same template 300 times:

* ≈ 5x times faster (mean) than [`project-init`](https://github.com/legion-labs/project-init)
    * 7x faster in the worst case
    * 5x faster in the best case

* ≈ 52x times faster (mean) than [`cookiecutter`](https://github.com/cookiecutter/cookiecutter)
    * 42x faster in the worst case
    * 60x faster in the best case

| Command                                      | Mean [ms] | Min [ms] | Max [ms] |
|:---------------------------------------------|---:|---:|---:|
| `tem new test project`                       | 5.9 ± 0.7 | 5.0 | 9.4 | 
| `pi new test project -f`                     | 28.6 ± 4.2 | 23.6 | 66.5 |
| `cookiecutter -f ~/temple/test2/ --no-input` | 330.2 ± 14.4 | 299.3 | 389.6 |

</br>

### 2. Easy set-up.

To create a new template, just drop the folder in `~/.temple` and create a `.temple` file where you can define project-level keys. To define keys, follow the syntax:

    key=Value with spaces,
    another_key=value,

Or with a single line (and how it should be done when defining keys with arguments):

    key=Value with spaces,another_key=value,

When defining keys there must not be more spaces than desired ones (key detection is primitive):

- Wrong: `key=Value with spaces ,  another_key=value,`
- Good:  `key=Value with spaces,another_key=value,`

- Wrong: 

```
key=Value with spaces , 
another_key=value
```
    
- Good:  
    
```
key=Value with spaces,
another_key=value
```


The hierarchy of keys is:

1. Command line defined (as argument) `temple new template name key=value,key2=Another value,key3=value`
2. Project defined `~/.temple/template_folder/.temple`
3. Global defined `~/.temple_conf`

</br>

### Non UTF-8 friendly

`temple` analyzes every file as a vector of bytes, only substitutions get converted to strings and panic when failed, so you can safely analyze executables and there shouldn't be any byte loses
