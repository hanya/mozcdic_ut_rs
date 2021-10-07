

This is a port of mozcdic_ut to Rust.

## How to make dictionary file

Execute make-dictionaries.sh file in src directory. Requires Rust >= 1.55.0.

```
cd src
./make-dictionaries.sh
```

Conversion of the dictionary file is done in build directory and
resulting mozcdic-ut-$UTDICDATE.txt file is created in the top directory.

If you execute single task, run as follows.

```
cd build
cargo run --release COMMAND [ FILENAME ]
```

List of valid commands can be seen by the following command.

```
cargo run --release help
```

## Compile mozc

Merge the dictionary file to mozc as follows and compile normally.

```
cat mozcdic-ut-$UTDICDATE.txt >> mozc-master/src/data/dictionary_oss/dictionary00.txt
```

## Dictionary contents

You can choose which dictionary is merged into a file. See make-dictionaries.sh file.

## Difference from the original mozcdic_ut

* Some dictionary output is little bit different from the original such as chimei.
* Filter output is little bit shorter than the original.
* No document output.
* No PKGBUILD update.
* No packaging.

The following files are copied from the mozcdic_ut.

* README.original.md
* src/make-dictionaries.sh
* data/*

## License

Placed under the same license as mozcdic_ut.
