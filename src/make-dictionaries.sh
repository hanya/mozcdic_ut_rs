#!/bin/bash

UTDICDATE="20210925"

#altcannadic="true"
chimei="true"
edict2="true"
jawikiarticles="true"
jinmeiut="true"
neologd="true"
#skkdic="true"
#sudachidict="true"


# ==============================================================================
# Make each dictionary
# ==============================================================================

mkdir -p ../build/
cd ../build/

rm -f ../mozcdic-ut-*.txt
rm -f ../build/mozcdic-*.txt*
rm -f ../build/jawiki-ut-*.txt

echo "Get the latest Mozc and make mozc-*.tar.bz2..."
cargo run --release mozc

echo "Get jawiki-titles and add search results to each title..."
cargo run --release jawikititles

echo "Generate alt-cannadic entries..."
cargo run --release altcannadic

echo "Generate chimei entries..."
cargo run --release chimei

echo "Generate edict2 entries..."
cargo run --release edict2

echo "Generate jawiki-articles entries..."
cargo run --release jawikiarticles
cargo run --release unsuitable mozcdic-ut-jawiki.txt

echo "Generate jinmei-ut entries..."
cargo run --release jinmeiut

echo "Generate neologd entries..."
cargo run --release neologd
cargo run --release unsuitable mozcdic-ut-neologd.txt

echo "Generate skkdic entries..."
cargo run --release skkdic

echo "Generate sudachidict entries..."
cargo run --release sudachidict
cargo run --release unsuitable mozcdic-ut-sudachidict-core.txt
cargo run --release unsuitable mozcdic-ut-sudachidict-notcore.txt


# ==============================================================================
# Extract new entries and calculate costs
# ==============================================================================

if [[ $altcannadic = "true" ]]; then
echo "Add alt-cannadic entries..."
cat mozcdic-ut-alt-cannadic*.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $chimei = "true" ]]; then
echo "Add chimei entries..."
cat mozcdic-ut-chimei.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $edict2 = "true" ]]; then
echo "Add edict2 entries..."
cat mozcdic-ut-edict2.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $jawikiarticles = "true" ]]; then
echo "Add jawiki-articles entries..."
cat mozcdic-ut-jawiki.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $jinmeiut = "true" ]]; then
echo "Add jinmei-ut entries..."
cat mozcdic-ut-jinmei.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $neologd = "true" ]]; then
echo "Add neologd entries..."
cat mozcdic-ut-neologd.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $skkdic = "true" ]]; then
echo "Add skkdic entries..."
cat mozcdic-ut-skkdic.txt >> mozcdic-ut-$UTDICDATE.txt
fi

if [[ $sudachidict = "true" ]]; then
echo "Add sudachidict entries..."
cat mozcdic-ut-sudachidict-*.txt >> mozcdic-ut-$UTDICDATE.txt
fi

echo "Remove unnecessary entries..."
cargo run --release unnecessary mozcdic-ut-$UTDICDATE.txt

echo "Calculate costs..."
cargo run --release costs mozcdic-ut-$UTDICDATE.txt.need

mv mozcdic-ut-$UTDICDATE.txt.need.costs ../mozcdic-ut-$UTDICDATE.txt
exit
echo "Update documents..."
cargo run --release document

echo "Copy mozc-*.tar.bz2 and PKGBUILD..."
cp -f ../build/mozc-*.tar.bz2 ../../
cp -f ../data/pkgbuild/*.PKGBUILD ../../


# ==============================================================================
# Make mozcdic-ut-*.tar.bz2
# ==============================================================================

echo "Copy files to mozcdic-ut-$UTDICDATE..."
cd ../../
rm -rf mozcdic-ut-$UTDICDATE
rsync -a mozcdic-ut-dev/* mozcdic-ut-$UTDICDATE --exclude=id.def --exclude=*.bzl \
--exclude=jawiki-latest* --exclude=jawiki-ut*.txt --exclude=KEN_ALL.* --exclude=*.csv \
--exclude=*.xml --exclude=*.gz --exclude=*.bz2 --exclude=*.xz --exclude=*.zip \
--exclude=*.html --exclude=_*.rb --exclude=*/mozcdic*.txt*

echo "Compress mozcdic-ut-$UTDICDATE..."
tar -cjf mozcdic-ut-$UTDICDATE.tar.bz2 mozcdic-ut-$UTDICDATE

echo "Done."
