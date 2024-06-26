git clean -X -f -d
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
cargo build --release
source ${EMSDK_DIR}/emsdk_env.sh
npm install -g typescript

pushd ./crates/raxb-libxml2-sys/third_party
mkdir -p ./libxml2-build
mkdir -p ./libxml2/m4
popd

pushd ./crates/raxb-libxml2-sys/third_party/libxml2
git clean -X -f
autoreconf -if -Wall
popd

pushd ./crates/raxb-libxml2-sys/third_party/libxml2-build
emconfigure ../libxml2/configure --disable-shared \
    --with-minimum --with-http=no --with-ftp=no --with-catalog=no \
    --with-python=no --with-threads=no \
    --with-output --with-c14n --with-zlib=no \
    --with-schemas --with-schematron
emmake make
popd
pushd ./packages/raxb-validate-wasm
./build.sh
popd