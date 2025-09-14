cd "$(dirname "$0")"
cd ..
NO_STRIP=true deno task tauri build
