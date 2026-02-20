# SQLite-vec Extension Setup

This document explains how to set up the sqlite-vec extension for RAG (Retrieval-Augmented Generation) features.

## What is sqlite-vec?

sqlite-vec is a SQLite extension that provides vector search capabilities. It's used by squid to store and query document embeddings for semantic search.

## Installation

### Option 1: Download Pre-built Extension (Recommended)

Download the sqlite-vec extension for your platform from the official releases:
https://github.com/asg017/sqlite-vec/releases

**macOS:**
```bash
# Download for macOS (ARM64/M1/M2/M3)
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-macos-arm64.tar.gz -o sqlite-vec.tar.gz
tar -xzf sqlite-vec.tar.gz
mkdir -p ~/.squid/extensions
mv vec0.dylib ~/.squid/extensions/
```

**Linux:**
```bash
# Download for Linux (x86_64)
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-linux-x86_64.tar.gz -o sqlite-vec.tar.gz
tar -xzf sqlite-vec.tar.gz
mkdir -p ~/.squid/extensions
mv vec0.so ~/.squid/extensions/
```

**Windows:**
```powershell
# Download for Windows (x86_64)
curl -L https://github.com/asg017/sqlite-vec/releases/latest/download/sqlite-vec-windows-x86_64.zip -o sqlite-vec.zip
Expand-Archive sqlite-vec.zip -DestinationPath .
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.squid\extensions"
Move-Item vec0.dll "$env:USERPROFILE\.squid\extensions\"
```

### Option 2: Build from Source

If you prefer to build from source or need a different platform:

```bash
git clone https://github.com/asg017/sqlite-vec.git
cd sqlite-vec
make loadable
# Copy the resulting extension to ~/.squid/extensions/
```

## Usage

Squid will automatically load the sqlite-vec extension from:
- `~/.squid/extensions/vec0.{dylib,so,dll}`
- Or from `SQUID_VEC_EXTENSION_PATH` environment variable if set

The extension is loaded when:
1. Running `squid serve` (for web UI)
2. Running `squid rag` commands (for indexing/querying)

## Troubleshooting

### Extension not found
```
Error: Failed to load sqlite-vec extension
```

**Solution:** Download the extension and place it in `~/.squid/extensions/` as shown above.

### Wrong architecture
```
Error: dlopen failed: wrong architecture
```

**Solution:** Download the correct version for your CPU architecture (ARM64 vs x86_64).

### Permission denied
```
Error: Permission denied loading extension
```

**Solution:** Ensure the extension file has execute permissions:
```bash
chmod +x ~/.squid/extensions/vec0.*
```

## Verification

To verify the extension is working:

```bash
squid rag stats
```

If successful, you'll see RAG statistics. If the extension isn't loaded, you'll see an error message.

## More Information

- sqlite-vec GitHub: https://github.com/asg017/sqlite-vec
- Documentation: https://alexgarcia.xyz/sqlite-vec/
