cd "$(dirname "$0")"
cd ..
NO_STRIP=true deno task tauri build
rm -rf out/
mkdir -p out/
cp src-tauri/target/release/simple-proton-save-locator out/
cp src-tauri/target/release/bundle/appimage/simple-proton-save-locator.AppDir/simple-proton-save-locator.png out/
cp src-tauri/target/release/bundle/appimage/simple-proton-save-locator.AppDir/simple-proton-save-locator.desktop out/