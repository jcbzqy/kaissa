```bash
# install mamba first
mamba create -n kaissa
mamba activate kaissa
mamba install clangxx clang-tools cmake conan libcxx libcxx-devel make ninja
cd cpp
mkdir -p build
conan install . --build=missing --profile conan_profiles/clang-linux-release
conan build . --profile conan_profiles/clang-linux-release
```