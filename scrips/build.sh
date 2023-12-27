#!/usr/bin/bash

if [[ -n $target ]]; then
    targets=($target)
else
    targets=("x86_64-unknown-linux-musl" "aarch64-unknown-linux-gnu" "x86_64-pc-windows-gnu")
    # "x86_64-apple-darwin" "aarch64-apple-darwin"
fi

IFS="= "
while read -r name value; do
    if [[ $name == "version" ]]; then
        version=${value//\"/}
    fi
done < Cargo.toml

echo "Compile mailpeter \"$version\""
echo ""

for target in "${targets[@]}"; do
    echo "compile static for $target"
    echo ""

    if [[ $target == "x86_64-pc-windows-gnu" ]]; then
        if [[ -f "mailpeter-v${version}_${target}.zip" ]]; then
            rm -f "mailpeter-v${version}_${target}.zip"
        fi

        cargo build --release --target=$target

        cp ./target/${target}/release/mailpeter.exe .
        zip -r "mailpeter-v${version}_${target}.zip" assets LICENSE README.md mailpeter.exe
        rm -f mailpeter.exe
    elif [[ $target == "x86_64-apple-darwin" ]] || [[ $target == "aarch64-apple-darwin" ]]; then
        if [[ -f "mailpeter-v${version}_${target}.tar.gz" ]]; then
            rm -f "mailpeter-v${version}_${target}.tar.gz"
        fi
        c_cc="x86_64-apple-darwin20.4-clang"
        c_cxx="x86_64-apple-darwin20.4-clang++"

        if [[ $target == "aarch64-apple-darwin" ]]; then
            c_cc="aarch64-apple-darwin20.4-clang"
            c_cxx="aarch64-apple-darwin20.4-clang++"
        fi

        CC="$c_cc" CXX="$c_cxx" cargo build --release --target=$target

        cp ./target/${target}/release/mailpeter .
        tar -czvf "mailpeter-v${version}_${target}.tar.gz" assets LICENSE README.md mailpeter
        rm -f mailpeter
    else
        if [[ -f "mailpeter-v${version}_${target}.tar.gz" ]]; then
            rm -f "mailpeter-v${version}_${target}.tar.gz"
        fi

        cargo build --release --target=$target

        cp ./target/${target}/release/mailpeter .
        tar -czvf "mailpeter-v${version}_${target}.tar.gz" assets LICENSE README.md mailpeter
        rm -f mailpeter
    fi

    echo ""
done

if [[ "${#targets[@]}" == "3" ]] || [[ $targets == "x86_64-unknown-linux-musl" ]]; then
    cargo deb --target=x86_64-unknown-linux-musl -o mailpeter_${version}-1_amd64.deb
    cargo generate-rpm --target=x86_64-unknown-linux-musl -o mailpeter-${version}-1.x86_64.rpm
fi

if [[ "${#targets[@]}" == "3" ]] || [[ $targets == "aarch64-unknown-linux-gnu" ]]; then
    cargo deb --target=aarch64-unknown-linux-gnu --variant=arm64 -o mailpeter_${version}-1_arm64.deb
fi
