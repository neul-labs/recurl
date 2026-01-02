# rcurl Documentation

User-facing documentation built with [MkDocs](https://www.mkdocs.org/) and [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).

## Development

### Prerequisites

```bash
# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate  # Linux/macOS
# or: venv\Scripts\activate  # Windows

# Install dependencies
pip install -r requirements.txt
```

### Local Development

```bash
# Serve locally with hot reload
mkdocs serve

# Open http://127.0.0.1:8000
```

### Build Static Site

```bash
# Build to site/ directory
mkdocs build

# Output in site/
```

## Structure

```
documentation/
├── mkdocs.yml              # MkDocs configuration
├── requirements.txt        # Python dependencies
├── README.md              # This file
└── docs/                  # Markdown source files
    ├── index.md           # Home page
    ├── getting-started/   # Getting started guides
    │   ├── quickstart.md
    │   ├── installation.md
    │   └── first-request.md
    ├── usage/             # Usage documentation
    │   ├── cli.md
    │   ├── modes.md
    │   └── environment.md
    ├── how-it-works/      # Technical documentation
    │   ├── architecture.md
    │   ├── layers.md
    │   ├── detection.md
    │   └── daemon.md
    └── reference/         # Reference documentation
        ├── platforms.md
        ├── antibot-services.md
        └── troubleshooting.md
```

## Deployment

### GitHub Pages

```bash
# Deploy to GitHub Pages
mkdocs gh-deploy
```

### Manual Deployment

```bash
# Build and upload site/ directory to web server
mkdocs build
rsync -avz site/ user@server:/var/www/rcurl-docs/
```

## Customization

### Theme

The documentation uses Material for MkDocs with:
- Light/dark mode toggle
- Deep purple primary color
- Search functionality
- Code copy buttons
- Navigation tabs

Configuration in `mkdocs.yml`.

### Adding Pages

1. Create markdown file in `docs/`
2. Add to `nav` section in `mkdocs.yml`

### Markdown Extensions

Available extensions:
- Admonitions (`!!! note`, `!!! warning`, etc.)
- Code highlighting with copy button
- Tabbed content
- Tables
- Permalinks on headings
