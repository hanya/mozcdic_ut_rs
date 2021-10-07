# Maintainer: UTUMI Hirosi <utuhiro78 at yahoo dot co dot jp>
# Contributor: Jiachen Yang <farseerfc@archlinux.org>
# Contributor: Felix Yan <felixonmars@gmail.com>
# Contributor: ponsfoot <cabezon dot hashimoto at gmail dot com>

# Mozc compile option
_bldtype=Release

_mozcver=2.26.4507.102
_fcitxver=20210906
_iconver=20210520
_utdicver=20210925
pkgver=${_mozcver}.${_utdicver}
pkgrel=1

_pkgbase=mozc
pkgname=fcitx5-mozc-ut
pkgdesc="Fcitx5 Module of A Japanese Input Method for Chromium OS, Windows, Mac and Linux (the Open Source Edition of Google Japanese Input)"
arch=('x86_64')
url="https://osdn.net/users/utuhiro/pf/utuhiro/files/"
license=('custom')
depends=('fcitx5' 'qt5-base')
makedepends=('clang' 'gyp' 'ninja' 'pkg-config' 'python' 'curl' 'qt5-base' 'fcitx5' 'libxcb' 'glib2' 'bzip2' 'unzip')
conflicts=('fcitx5-mozc')

source=(
  https://osdn.net/users/utuhiro/pf/utuhiro/dl/mozc-${_mozcver}.tar.bz2
  abseil-cpp-20210324.1.tar.gz::https://github.com/abseil/abseil-cpp/archive/refs/tags/20210324.1.tar.gz
  googletest-release-1.10.0.tar.gz::https://github.com/google/googletest/archive/release-1.10.0.tar.gz
  japanese-usage-dictionary-master.zip::https://github.com/hiroyuki-komatsu/japanese-usage-dictionary/archive/master.zip
  protobuf-3.13.0.tar.gz::https://github.com/protocolbuffers/protobuf/archive/v3.13.0.tar.gz
  https://osdn.net/users/utuhiro/pf/utuhiro/dl/fcitx5-mozc-${_fcitxver}.patch
  https://osdn.net/users/utuhiro/pf/utuhiro/dl/fcitx5-mozc-icons-${_iconver}.tar.gz
  https://osdn.net/users/utuhiro/pf/utuhiro/dl/mozcdic-ut-${_utdicver}.tar.bz2
  https://www.post.japanpost.jp/zipcode/dl/kogaki/zip/ken_all.zip
  https://www.post.japanpost.jp/zipcode/dl/jigyosyo/zip/jigyosyo.zip
)

sha256sums=(
  'ab35c19efbae45b1fbd86e61625d4d41ad4fb95beefdf5840bdd7ee2f7b825cd'
  '441db7c09a0565376ecacf0085b2d4c2bbedde6115d7773551bc116212c2a8d6'
  '9dc9157a9a1551ec7a7e43daea9a694a0bb5fb8bec81235d8a1e6ef64c716dcb'
  'e46b1c40facbc969b7a4af154dab30ab414f48a0fdbe57d199f912316977ac25'
  '9b4ee22c250fe31b16f1a24d61467e40780a3fbb9b91c3b65be2a376ed913a1a'
  '0dbe8c94f4ee1bc41ef8418ee1830d97bd54b05b9d9807ac3163bd15cc4198a5'
  '4ebaf2d3ef8029a0fb40fce600471876d4bcd6492f99c083e5aa5b221614e4e4'
  'SKIP'
  'SKIP'
  'SKIP'
)

prepare() {
  cd mozc-${_mozcver}
  rm -rf src/third_party
  mkdir src/third_party
  mv ${srcdir}/abseil-cpp-20210324.1 src/third_party/abseil-cpp
  mv ${srcdir}/googletest-release-1.10.0 src/third_party/gtest
  mv ${srcdir}/japanese-usage-dictionary-master src/third_party/japanese_usage_dictionary
  mv ${srcdir}/protobuf-3.13.0 src/third_party/protobuf
  patch -Np1 -i ${srcdir}/fcitx5-mozc-${_fcitxver}.patch

  # Add ZIP code
  cd src/data/dictionary_oss/
  PYTHONPATH="${PYTHONPATH}:../../" \
  python ../../dictionary/gen_zip_code_seed.py \
  --zip_code=${srcdir}/KEN_ALL.CSV --jigyosyo=${srcdir}/JIGYOSYO.CSV >> dictionary09.txt
  cd -

  # Generate aux_dictionary.txt
  cd src/data/oss/
  PYTHONPATH="${PYTHONPATH}:../../" \
  python ../../dictionary/gen_aux_dictionary.py \
  --output aux_dictionary.txt \
  --aux_tsv aux_dictionary.tsv \
  --dictionary_txts ../../data/dictionary_oss/dictionary0*.txt
  cat aux_dictionary.txt >> ../../data/dictionary_oss/dictionary09.txt
  cd -

  # Use libstdc++ instead of libc++
  sed "/stdlib=libc++/d;/-lc++/d" -i src/gyp/common.gypi

  # Add UT dictionary
  cat ${srcdir}/mozcdic-ut-${_utdicver}/mozcdic-ut-${_utdicver}.txt >> src/data/dictionary_oss/dictionary00.txt
}

build() {
  cd mozc-${_mozcver}/src

  _targets="server/server.gyp:mozc_server gui/gui.gyp:mozc_tool unix/fcitx5/fcitx5.gyp:fcitx5-mozc"

  GYP_DEFINES="enable_gtk_renderer==0" python build_mozc.py gyp --gypdir=/usr/bin --target_platform=Linux
  python build_mozc.py build -c $_bldtype $_targets
}

package() {
  cd mozc-${_mozcver}/src
  install -D -m 755 out_linux/${_bldtype}/mozc_server ${pkgdir}/usr/lib/mozc/mozc_server
  install -m 755 out_linux/${_bldtype}/mozc_tool ${pkgdir}/usr/lib/mozc/mozc_tool

  install -d ${pkgdir}/usr/share/licenses/$pkgname/
  install -m 644 ../LICENSE data/installer/*.html ${pkgdir}/usr/share/licenses/${pkgname}/

  for pofile in unix/fcitx5/po/*.po
  do
      filename=`basename $pofile`
      lang=${filename/.po/}
      mofile=${pofile/.po/.mo}
      msgfmt $pofile -o $mofile
      install -D -m 644 $mofile ${pkgdir}/usr/share/locale/$lang/LC_MESSAGES/fcitx5-mozc.mo
  done

  install -D -m 755 out_linux/${_bldtype}/fcitx5-mozc.so ${pkgdir}/usr/lib/fcitx5/fcitx5-mozc.so
  install -D -m 644 unix/fcitx5/mozc-addon.conf ${pkgdir}/usr/share/fcitx5/addon/mozc.conf
  install -D -m 644 unix/fcitx5/mozc.conf ${pkgdir}/usr/share/fcitx5/inputmethod/mozc.conf

  msgfmt --xml -d unix/fcitx5/po/ --template unix/fcitx5/org.fcitx.Fcitx5.Addon.Mozc.metainfo.xml.in -o unix/fcitx5/org.fcitx.Fcitx5.Addon.Mozc.metainfo.xml
  install -D -m 644 unix/fcitx5/org.fcitx.Fcitx5.Addon.Mozc.metainfo.xml ${pkgdir}/usr/share/metainfo/org.fcitx.Fcitx5.Addon.Mozc.metainfo.xml

  install -d ${pkgdir}/usr/share/doc/${pkgname}/
  cp {../AUTHORS,../LICENSE,../README.md} ${pkgdir}/usr/share/doc/${pkgname}/

  # Install icons
  # https://github.com/fcitx/mozc/blob/fcitx/scripts/install_fcitx5_icons
  install -d ${pkgdir}/usr/share/icons/
  cp -r ${srcdir}/fcitx5-mozc-icons-${_iconver}/* ${pkgdir}/usr/share/icons/
}
