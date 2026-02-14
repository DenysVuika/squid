# Web Development Guide

This document explains how to develop and build the Squid Web UI.

## Architecture

The Squid Web UI is a React application built with Vite that connects to the Rust backend server.

**Development Mode:**
- Frontend: Vite dev server runs on `http://localhost:5173`
- Backend: Rust server runs on `http://127.0.0.1:8080`
- Vite proxy forwards `/api` requests to the Rust server

**Production Mode:**
- The Rust server serves the built frontend static files from the `static/` directory
- All API requests use relative paths (same origin)

## Development Workflow

### 1. Start the Rust Backend Server

```bash
# From project root
cargo run serve --port 8080
```

The server will start on `http://127.0.0.1:8080` and serve:
- API endpoints at `/api/*`
- Static files (in production) at `/`

### 2. Start the Vite Dev Server

In a separate terminal:

```bash
cd web
npm run dev
```

The Vite dev server will start on `http://localhost:5173` with:
- Hot module replacement (HMR) for instant updates
- API proxy to `http://127.0.0.1:8080`

### 3. Open in Browser

Navigate to `http://localhost:5173` to see the app with hot reloading.

All API requests (e.g., `/api/chat`, `/api/sessions`) are automatically proxied to the Rust server.

## Building for Production

### Build the Frontend

```bash
cd web
npm run build
```

This compiles the React app and outputs to `../static/` directory.

### Run the Production Server

```bash
# From project root
cargo run serve --port 8080
```

The server now serves the built frontend at `http://127.0.0.1:8080`.

## Configuration

### Vite Proxy (Development)

The Vite dev server proxies API requests to the Rust server. Configuration in `web/vite.config.ts`:

```typescript
server: {
  port: 5173,
  proxy: {
    '/api': {
      target: 'http://127.0.0.1:8080',
      changeOrigin: true,
    },
  },
}
```

### CORS (Development)

The Rust server allows CORS for development mode, enabling the Vite dev server to make requests.

## Common Tasks

### Add a New Component

```bash
cd web/src/components/app
# Create your component file
```

Components in `web/src/components/app/` are application-specific and can be freely modified.

**Do not modify** components in `web/src/components/ai-elements/` unless fixing bugs.

### Update API Endpoints

1. Add endpoint to `src/api.rs`
2. Update function in `web/src/lib/chat-api.ts`
3. Use the function in your component/store

### Debug API Requests

Watch the Rust server logs:

```bash
# Terminal 1: Rust server (see API logs)
cargo run serve --port 8080

# Terminal 2: Vite dev server
cd web && npm run dev
```

Check browser DevTools Network tab to see proxied requests.

## Troubleshooting

### Port Already in Use

If port 8080 or 5173 is already in use:

```bash
# Change Rust server port
cargo run serve --port 8081

# Update proxy target in web/vite.config.ts
target: 'http://127.0.0.1:8081'
```

### CORS Errors

If you see CORS errors in the browser console, ensure:
1. The Rust server is running
2. The `actix-cors` middleware is configured in `src/main.rs`
3. The Vite proxy target matches the Rust server address

### Build Errors

```bash
# Clean and rebuild
cd web
rm -rf node_modules dist
npm install
npm run build
```

### Hot Reload Not Working

1. Check that Vite dev server is running on `http://localhost:5173`
2. Restart the Vite dev server
3. Clear browser cache

## File Structure

```
web/
├── src/
│   ├── components/
│   │   ├── ai-elements/  # Reusable UI components (DO NOT MODIFY)
│   │   └── app/          # Application components (MODIFY FREELY)
│   ├── lib/
│   │   └── chat-api.ts   # API client functions
│   ├── stores/           # Zustand state stores
│   └── App.tsx           # Main app component
├── vite.config.ts        # Vite configuration + proxy
└── package.json          # Dependencies and scripts
```

## Tips

- Use `console.log()` liberally during development
- Check both Rust server logs and browser console for errors
- The Vite dev server has better error messages than the production build
- Use browser DevTools to inspect API requests and responses
