# Temple

Fast experimental project template renderer with easy setup and usage.

## Installation

	cargo install --git https://github.com/AOx0/temple

##  Why temple?

### 1. It's fast.

It is fast, very fast. As shown in the benchmark results, where each tool rendered the same template 1500 times:

| Command | Mean ms | Min ms | Max ms | Relative |
|:---|---:|---:|---:|---:|
| `tnew test test` | 9.4 ± 1.1 | 8.0 | 19.3 | 1.00 |
| `pi new test test -f` | 38.4 ± 4.5 | 32.4 | 64.8 | 4.07 ± 0.67 |
| `cookiecutter test --no-input -f` | 410.9 ± 96.5 | 325.0 | 1373.8 | 43.51 ± 11.38 |

	Summary
	  'tnew test test' ran
	    4.07 ± 0.67 times faster than 'pi new test test -f'
	   43.51 ± 11.38 times faster than 'cookiecutter test --no-input -f'

### 2. Easy set-up.

To create a new template, just drop the folder in `~/.temple` and create a `.temple` file where you can define project-level keys. To define keys, follow the syntax:

	key=Value with spaces,
	another_key=value,

Or with a single line (and how it should be done when defining keys with arguments):

	key=Value with spaces,another_key=value,

You can define custom key-indicators. By default its `{{ ` for `start_indicator ` and ` }}` for `start_indicator `,

You can set them as following:

	start_indicator=[[[,
	end_indicator=]]]

When defining keys there must not be more spaces than desired ones (key detection is primitive):

- Wrong: `key=Value with spaces ,  another_key=value,`
- Good:  `key=Value with spaces,another_key=value,`

- Wrong: 

		key=Value with spaces , 
		another_key=value

- Good:  

		key=Value with spaces,
		another_key=value


The hierarchy of keys is:

1. Command line defined (as argument) `temple new template name key=value,key2=Another value,key3=value`
2. Project defined `~/.temple/template_folder/.temple`
3. Global defined `~/.temple_conf`

\<br/\>

### Non UTF-8 friendly

`temple` analyzes every file as a vector of bytes, only substitutions get converted to strings and panic when failed, so you can safely analyze executables and there shouldn't be any byte loses

