# kaissa

basic UCI engines  
separate engines in cpp and rust  
array (not bitboard) representation

## cpp
```bash
# install mamba first
mamba create -n kaissa
mamba activate kaissa
mamba install clangxx clang-tools cmake conan libcxx libcxx-devel make ninja
cd cpp
mkdir -p build
conan install . --build=missing --profile conan_profiles/clang-linux-release
conan build . --profile conan_profiles/clang-linux-release
./build/Release/kaissa_tests
```

## rust
```bash
# install gcc, rustup first
cd rust
cargo build --release
cargo test
```