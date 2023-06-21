# FindFile (FF)
An simple, ergonomic, and powerful replacement for `find`.

<h1>Note: this repo is under active development</h1>
The syntax is (mostly) figured out, but the internals and implementations of the syntax need a lot of work.

## Road Map
- [x] Basic Lexer
	- [x] string literals (with interpolation): `"..."`
	- [x] perl regex literals (with interpolation): `$/.../`
	- [x] path literals (sorta with interpolation): `foo/*.txt`
	- [x] filesize literals: `12kb`, `4.9mib`
	- [x] `$env` vars and `$1` cli vars
	- [ ] date & time literals
	- [ ] have `+` in path literals start at the search root, not always at pwd root.
- [~] Basic AST Builder
	- [x] math & logic binary operators (most dont work in the runtime)
	- [x] blocks of code
	- [x] basic assignment
	- [ ] `-Xk` and `+Xk` need to be implemented for larger & smaller
	- [ ] compound assignment
	- [ ] logical assignment
	- [ ] arrays & hashmaps (they'll be the same, type)
	- [ ] function calls
	- [ ] function declarations
- [ ] Basic Runtime
	- [x] Figure out starting position (mostly works)
	- [ ] Add in basic math for most types
	- [ ] Cleanup how variables are accessed
	- [ ] Support `^{}` and `${}` for begin and end blocks
	- [ ] A way to convert to and from different types
	- [ ] Add more supported functions
		- [ ] Fill out the ones already in this file
		- [ ] Addd `depth` and "amount of children in directory"
	- [ ] Convert it to a vm
		- [ ] add in a JIT
- [x] Argument Parser
	- [x] Most arguments (both currently implemented and todos) are added
	- [ ] An option to print out matched lines in their files (a-la ripgrep)
	- [ ] Clean it up to make it look really pretty (clap mostly does a good job)
- [ ] Misc
	- [ ] Optimize file reading so you dont read an entire file to math the first line
	- [ ] cleanup type represenations
	- [ ] maybe remove the significant bits from filesizes?
	- [ ] Solidify when I'm using `Vec<u8>` vs `OsString` vs `String`.
	- [ ] Figure out how to reconcile `${}` for begin blocks and `${}` for env vars
	- [ ] should unknown variables warn?

Here's some examples of things I want to eventually support
- list all files in a directory: ``ff 'isfile && depth=1' ``
- make a "tree" of files and their directories:	``ff -n 'print "\t"*depth_from(start), basename'``
- find all files that're at least 1 gig or are newer than 10 days ago: ``ff 'size > 1g || modify > -10d'``
- add the suffix `-YYYY-MM-DD` to all files but keep the extension: ``ff -n 'isfile && mv(file, "{dir}{base}-{ymd_date}.{suffix})'``
- find files newer than 10 days with the enclosing folder is `log`: ``ff 'isfile && modify > -10d && basename(parent) = "log"'``
- find all files that contain "hello" and "world", possibly on separate lines: ``ff 'contents =~ /hello/ && contents =~ /world/'``
- find the largest folder by its immediate files: (`${}` is run at script end): ``ff -n '${print maxdir} dirsize > dirsize(maxdir) then maxdir=dirsize'``

Keywords
---

- `true` equivalent to `1`
- `false` equivalent to `0`

if else while continue break def return `skip`/`next`

# Functions
If a function takes no arguments, you can just omit the parens. eg `file?` is the same as `file?()`,
as `file?()` is equivalent to `file?(path)`.

## Querying Info
|    name and args    |   aliases   | what it does |
|---------------------|-------------|--------------|
| `file?(p=path)`       | `f?`        | Returns whether `p` is a file. |
| `directory?(p=path)`  | `d?` `dir?` | Returns whether `p` is a directory. |
| `executable?(p=path)` | `e?` `exe?` | Returns whether `p` is an executable. |
| `symlink?(p=path)`    | `s?` `sym?` | Returns whether `p` is a symlink. |
| `binary?(p=path)`     | `b?` `bin?` | Returns whether `p` is a binary file. |
| `gitignore?(p=path)`  | `gi?`       | Returns whether `p` is ignored by a gitignore file. |
| `hidden?(p=path)`     | `gi?`       | Returns whether `p` is starts with `.` |
| `ok?(msg)`          |             | Prints `msg` out, then asks for confirmation. |
| `macos(...)`        |             | future idea: stuff like macos tags or whatnot |

## Path-Related functions
| `root()` | `r` | The root folder we started looking at |
| `path()` | `p` | The current path |
| `dirname(p=path)` | `d` `dir` `parent` | The parent directory |
| `extname(p=path)` | `e` `ext` `extension` | The extension, without a `.` if it's present |
| `extnamed(p=path)` | `ed` `extd` `extensiond` | The extension, including the `.` if it's present. |
| `basename(p=path)` | `b` `bn` `base` | Everything but the parent directory of `p` |
| `stemname(p=path)` | `s` `stem` | `basename`, except without an extension (if present) |


## Misc
| `print(...)`        | `pr` | Prints its arguments out (with nothing between them) followed by a newline |
| `printn(...)`       | `prn` | Prints its arguments out (with nothing between them) without a newline |
| `next` | `skip` | Ignores the current argument and continues onwards |
| `exit(status)` | `quit` | stops the entire script |
| `pwd()`       | Current working directory |
| `depth(src=path, dst=root)` | how many directories down we are from the `dst`. |
| `date(<...>)` | The current date foromatted in a time |
| `sleep(<...>)` | Sleeps |

## Executable functions
Some of these functions are "destructive" (such as `mv`): If a destructive file would overwrite another one, it'll check the command line arguments to see what to do (`--interactive` implies always ask, `--force` implies never ask; if neither is given, `--force` is assumed.) You can use `<fn>i` to always do interactive or `<fn>f` to always force (like `mvf`).

All these must be called with parens (maybe?)

|   name and args    | what it does |
|--------------------|-------------|
| exec(...) |..| todo, i dont like how exec is normally done in everything else
| `mv{,f,i}(src=path, dst)` | Moves `src` to `dst`; only confirms if overwriting a file when interactive |
| `rm{,f,i}(src=path)`      | Removes the file at `src`; always confirms when interactive. If given an empty directory, `rm` acts like `rmdir`. |
| `rmr{,f,i}(src=path)`     | Removes the file at `src`, recursively; always confirms when interactive  |
| `cp{,f,i}(src=path, dst)` | Copies `src` to `dst` only confirms if overwriting a file when interactive |
| `ln...`||
| `mkdir(p)` | Creates a directory at `p`; It'll also make all parent directories. |
| `touch(src=path)` | Creates a directory at `p`; It'll also make all parent directories. |



## IDEAS:
there was a problem with moving your file. What would you like to do:
(Q)uit: Stop the entire program
(C)ontinue: continue onwards,
(R)etry: try it again (maybe after you fix something)
(S)hell: Drop you into a shell where `$cpath` is the variable for the current path
