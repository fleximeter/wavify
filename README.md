# wavify
Automatically converts non-WAV audio in a directory and its subdirectories to WAV format. It gives the new files the same name (with the wav file extension), and puts them in the same subdirectory. You can optionally delete the original files.

## Optional command-line parameters:
-f, --folder: The directory to convert. Defaults to "."
-n, --num-threads: The maximum number of threads to run. If 0 (default), the program uses the same number of threads as there are processors available.

## Optional command-line flag
-d, --delete: If you provide this flag, the original files will be deleted.
