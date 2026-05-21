# Cloudflare Pages Deployment Guide

This guide explains how to deploy the mml2vgm Browser IDE to Cloudflare Pages.

## Prerequisites

1. A Cloudflare account (free tier is sufficient)
2. GitHub repository connected to Cloudflare
3. Node.js v18+ installed (for local testing)

## Quick Deployment via Cloudflare Dashboard

### Step 1: Connect GitHub Repository

1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Select your account
3. Click **Pages** in the left sidebar
4. Click **Create application** > **Connect GitHub account**
5. Authorize Cloudflare to access your GitHub repositories
6. Select the `mml2vgm` repository

### Step 2: Configure Build Settings

| Setting | Value |
|---------|-------|
| **Project name** | `mml2vgm-browser-ide` (or any name you prefer) |
| **Production branch** | `master` (or your main branch) |
| **Build command** | `cd browser-ide && npm install && npm run build` |
| **Build output directory** | `browser-ide/dist` |
| **Root directory** | (leave empty for root) |

### Step 3: Configure Environment Variables

Add the following environment variables (optional, for production):

| Variable | Value | Required |
|----------|-------|----------|
| `NODE_VERSION` | `18` | Yes |
| `BASE_PATH` | `/` | No (defaults to `/`) |

### Step 4: Deploy

1. Click **Save and Deploy**
2. Wait for the build to complete (typically 2-5 minutes)
3. Once deployed, you'll get a URL like `https://mml2vgm-browser-ide.pages.dev`

---

## Advanced: Wrangler CLI Deployment

For more control, you can deploy using the Wrangler CLI.

### Step 1: Install Wrangler

```bash
npm install -g wrangler
```

### Step 2: Authenticate

```bash
wrangler login
```

### Step 3: Deploy

```bash
cd browser-ide
wrangler pages deploy dist --project-name mml2vgm-browser-ide
```

---

## Configuration Files

### `_redirects` (in `browser-ide/public/`)

For Single Page Application (SPA) routing, Cloudflare Pages automatically handles client-side routing. The `_redirects` file ensures all paths serve `index.html`:

```
/*    /index.html    200
```

### Headers Configuration

Required headers for WASM support are configured in `cloudflare-pages.toml`:

```toml
# Required for SharedArrayBuffer (needed for WASM AudioWorklet)
[[headers]]
  for = "/"
  [headers.values]
    Cross-Origin-Opener-Policy = "same-origin"
    Cross-Origin-Embedder-Policy = "require-corp"

# Cache WASM files aggressively
[[headers]]
  for = "/wasm/*"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000, immutable"
```

**Note:** These headers can also be configured through the Cloudflare Dashboard under Pages > your project > Settings > Headers.

---

## Custom Domain Setup

To use a custom domain (e.g., `ide.mml2vgm.com`):

1. Go to your Pages project in Cloudflare Dashboard
2. Click **Custom domains**
3. Click **Set up a custom domain**
4. Enter your domain (must be managed by Cloudflare DNS)
5. Cloudflare will automatically configure SSL and DNS

---

## Build Output Structure

After running `npm run build`, the `dist/` directory contains:

```
dist/
├── index.html
├── index.css
├── _redirects
├── wasm/
│   ├── pkg/
│   │   ├── mml2vgm_wasm.js
│   │   ├── mml2vgm_wasm_bg.wasm
│   │   └── mml2vgm_wasm.d.ts
│   └── ...
├── public/
│   ├── locales/
│   │   ├── en.json
│   │   └── ja.json
│   └── sw.js
└── assets/
    └── ... (compiled JS and CSS)
```

---

## Development Workflow

### Local Development

```bash
cd browser-ide
npm install
npm run dev
```

Access at: `http://localhost:5173`

### Preview Production Build

```bash
cd browser-ide
npm run build
npm run preview
```

Access at: `http://localhost:4173`

---

## Troubleshooting

### Build Fails with TypeScript Errors

The test files use vitest-specific globals. If you see errors like:
```
error TS1005: '>' expected
```

This is expected. The test files are configured separately with `vitest.tsconfig.json`. For the main build, these files are not included in the production bundle.

### WASM Files Not Loading

Ensure the following headers are set:
- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

These are required for SharedArrayBuffer, which is needed for the WASM audio playback.

### Service Worker Not Registering

Check the browser console for errors. The service worker (`public/sw.js`) should register automatically on page load. In production, ensure:
1. The site is served over HTTPS (Cloudflare Pages provides this automatically)
2. The service worker path is correct

### Offline Mode Not Working

1. Visit the site at least once to cache assets
2. Check the Application > Service Workers tab in DevTools
3. Verify the cache is populated (Application > Cache Storage)

---

## CI/CD with GitHub Actions

If you prefer GitHub Actions instead of Cloudflare's built-in CI:

### `.github/workflows/deploy.yml`

```yaml
name: Deploy to Cloudflare Pages

on:
  push:
    branches: [master]

jobs:
  deploy:
    runs-on: ubuntu-latest
    name: Deploy
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          
      - name: Install dependencies
        run: |
          cd browser-ide
          npm install
          
      - name: Build
        run: |
          cd browser-ide
          npm run build
          
      - name: Deploy to Cloudflare Pages
        uses: cloudflare/pages-action@1
        with:
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          projectName: mml2vgm-browser-ide
          directory: browser-ide/dist
          # Optional: Only deploy if build succeeded
          gitHubToken: ${{ secrets.GITHUB_TOKEN }}
```

You'll need to set these GitHub Secrets:
- `CLOUDFLARE_API_TOKEN` - Create at: https://dash.cloudflare.com/profile/api-tokens
- `CLOUDFLARE_ACCOUNT_ID` - Your Cloudflare account ID

---

## Performance Optimization

Cloudflare Pages automatically provides:
- Global CDN distribution
- Automatic HTTPS
- Brotli compression
- Image optimization (for images in public/)

For additional optimization:
1. Enable **Auto Minify** in Cloudflare Dashboard > Workers & Pages > Settings
2. Consider using Cloudflare's **Speed** features

---

## Monitoring

After deployment, monitor your site:
1. Cloudflare Dashboard > Pages > your project > Analytics
2. Check the **Deployments** tab for build logs
3. Use the **Logs** tab for runtime errors (if using Functions)

---

## Resources

- [Cloudflare Pages Documentation](https://developers.cloudflare.com/pages/)
- [Cloudflare Pages API](https://api.cloudflare.com/#pages)
- [Vite Documentation](https://vitejs.dev/)
- [Vite + Cloudflare Pages](https://developers.cloudflare.com/pages/framework-guides/deploy-a-vite-react-site/)
