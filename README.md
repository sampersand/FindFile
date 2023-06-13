# FindFile (FF)
A replacement for `find` (and `fd`) that's simple to use, but much more powerful.

This repo is currently **under active development**, so it doesn't work at all. (Or, it's been abandoned and I forgot to update this README. Either or.)

Here's some examples of ways to do things in ff:

- list all files in a directory: ``ff 'isfile && depth=1' ``
- make a "tree" of files and their directories:	``ff -n 'print "\t"*depth_from(start), basename'``
- find all files that're at least 1 gig or are newer than 10 days ago: ``ff 'size > 1g || modify > -10d'``
- add the suffix `-YYYY-MM-DD` to all files but keep the extension: ``ff -n 'isfile && mv(file, "{dir}{base}-{ymd_date}.{suffix})'``
- find files newer than 10 days with the enclosing folder is `log`: ``ff 'isfile && modify > -10d && basename(parent) = "log"'``
- find all files that contain "hello" and "world", possibly on separate lines: ``ff 'contents =~ /hello/ && contents =~ /world/'``
- find the largest folder by its immediate files: (`${}` is run at script end): ``ff -n '${print maxdir} dirsize > dirsize(maxdir) then maxdir=dirsize'``