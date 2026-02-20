# SQLite-vec Extension Setup

This document explains the sqlite-vec integration in squid.

## How It Works

Squid uses the **sqlite-vec Rust crate** which automatically compiles and embeds the sqlite-vec extension at build time. **No manual installation required!**

The extension is registered using `sqlite3_auto_extension` when the database is initialized, so it's available in all database connections.

## For Developers

Just run:

```bash
cargo build
```

The sqlite-vec extension is compiled and statically linked automatically. RAG features work out of the box!

## Alternative: Dynamic Loading (Advanced)

If you're distributing pre-compiled binaries and want to avoid embedding sqlite-vec, you can optionally load it dynamically at runtime.

This requires:
1. Removing the `sqlite-vec` crate from dependencies
2. Re-implementing dynamic loading (see git history)
3. Users downloading the extension manually

### Manual Installation (for dynamic loading only)

**macOS:**
```bash
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-macos-arm64.tar.gz -o sqlite-vec.tar.gz
tar -xzf sqlite-vec.tar.gz
mkdir -p ~/.squid/extensions
mv vec0.dylib ~/.squid/extensions/
```

**Linux:**
```bash
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-linux-x86_64.tar.gz -o sqlite-vec.tar.gz
tar -xzf sqlite-vec.tar.gz
mkdir -p ~/.squid/extensions
mv vec0.so ~/.squid/extensions/
```

**Windows:**
```powershell
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-windows-x86_64.zip -o sqlite-vec.zip
Expand-Archive sqlite-vec.zip -DestinationPath .
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.squid\extensions"
Move-Item vec0.dll "$env:USERPROFILE\.squid\extensions\"
```

## Verification

To verify sqlite-vec is working:

```bash
cargo run -- serve
# Check logs for: "Registered sqlite-vec extension"
```

Or once RAG commands are implemented:

```bash
squid rag stats
```

## Troubleshooting

### Build fails with sqlite-vec compilation errors

Make sure you have a C compiler installed:
- **macOS**: `xcode-select --install`
- **Linux**: `sudo apt install build-essential`
- **Windows**: Install Visual Studio Build Tools

### RAG features not working

Check that:
1. The `sqlite-vec` crate is in `Cargo.toml`
2. You're using `rusqlite` with the `bundled` feature
3. The extension is registered in `src/db.rs`

## More Information

- sqlite-vec GitHub: https://github.com/asg017/sqlite-vec
- Rust integration: https://alexgarcia.xyz/sqlite-vec/rust.html
- Documentation: https://alexgarcia.xyz/sqlite-vec/
