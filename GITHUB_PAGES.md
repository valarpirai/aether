---
layout: default
title: GitHub Pages Deployment Guide
---

# GitHub Pages Deployment Guide

This guide explains how to enable and configure GitHub Pages for the Aether documentation site.

## Prerequisites

- Repository hosted on GitHub
- Documentation files in `docs/` directory
- Jekyll configuration (`_config.yml`) in `docs/` directory

## Enabling GitHub Pages

### Step 1: Configure Repository Settings

1. Go to your GitHub repository
2. Click on **Settings** tab
3. Scroll down to **Pages** section (left sidebar)
4. Under **Source**, select:
   - **Branch**: `main` (or your default branch)
   - **Folder**: `/docs`
5. Click **Save**

### Step 2: Wait for Deployment

- GitHub will automatically build and deploy your site
- First deployment typically takes 1-2 minutes
- You'll see a green checkmark and URL when ready
- URL format: `https://yourusername.github.io/repository-name/`

### Step 3: Update Configuration

Edit `docs/_config.yml` and update these fields:

```yaml
url: "https://yourusername.github.io/aether"
repository: yourusername/aether
```

Replace `yourusername` with your actual GitHub username.

## Site Structure

The documentation site is organized as follows:

```
docs/
├── _config.yml          # Jekyll configuration
├── index.md             # Landing page
├── DESIGN.md            # Language specification
├── ARCHITECTURE.md      # System architecture
├── DEVELOPMENT.md       # Development guide
├── TESTING.md           # Testing guide
├── LEXER.md            # Lexer documentation
├── PARSER.md           # Parser documentation
├── INTERPRETER.md      # Interpreter documentation
├── REPL.md             # REPL documentation
├── STDLIB.md           # Standard library
├── MODULE_SYSTEM.md    # Module system
├── STRUCT.md           # Structs feature
├── ERROR_HANDLING.md   # Error handling
├── STRING_FEATURES.md  # String features
├── JSON.md             # JSON support
├── TIME.md             # Time functions
├── HTTP.md             # HTTP functions
└── GC_DESIGN.md        # Garbage collection
```

## Navigation

The site uses a categorized navigation structure defined in `_config.yml`:

### Categories

1. **Getting Started**
   - Language Design
   - Architecture
   - Development Guide
   - Testing Guide

2. **Components**
   - Lexer, Parser, Interpreter
   - REPL, Standard Library
   - Module System

3. **Features**
   - Structs, Error Handling
   - String Features, JSON
   - Time, HTTP

4. **Internals**
   - GC Design

## Theme

The site uses the **Cayman** theme, which provides:
- Clean, modern design
- Responsive layout
- Syntax highlighting for code blocks
- GitHub repository integration

### Alternative Themes

To change the theme, edit `docs/_config.yml`:

```yaml
theme: jekyll-theme-minimal  # Minimalist theme
# or
theme: jekyll-theme-slate    # Dark theme
# or
theme: minima                # Default Jekyll theme
```

See [Jekyll Themes](https://pages.github.com/themes/) for more options.

## Syntax Highlighting

Code blocks are automatically highlighted using Rouge:

```aether
fn main() {
    let message = "Hello, Aether!"
    println(message)
}
```

Supported languages:
- `aether` - Aether code (falls back to similar syntax)
- `rust` - Rust code (interpreter implementation)
- `bash` - Shell commands
- `json` - JSON data
- `yaml` - YAML configuration

## Customization

### Adding New Pages

1. Create a new `.md` file in `docs/`
2. Add YAML front matter:

```yaml
---
layout: default
title: Your Page Title
---
```

3. Write your content in Markdown
4. Add to navigation in `_config.yml` if needed

### Custom CSS

Create `docs/assets/css/style.scss`:

```scss
---
---

@import "{{ site.theme }}";

/* Your custom styles here */
.custom-class {
  color: #0066cc;
}
```

### Custom Layout

Create `docs/_layouts/custom.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{ page.title }} | {{ site.title }}</title>
</head>
<body>
  {{ content }}
</body>
</html>
```

## Testing Locally

### Install Jekyll

```bash
# macOS
gem install jekyll bundler

# Ubuntu/Debian
sudo apt-get install ruby-full build-essential
gem install jekyll bundler
```

### Create Gemfile

Create `docs/Gemfile`:

```ruby
source "https://rubygems.org"
gem "github-pages", group: :jekyll_plugins
```

### Run Local Server

```bash
cd docs
bundle install
bundle exec jekyll serve
```

Visit `http://localhost:4000` in your browser.

### Watch for Changes

Jekyll automatically rebuilds when files change:

```bash
bundle exec jekyll serve --watch
```

## Troubleshooting

### Build Failures

Check the **Actions** tab in your GitHub repository:
- Click on the failed workflow
- Review build logs for errors
- Common issues: invalid YAML, missing files, broken links

### 404 Errors

- Verify `baseurl` and `url` in `_config.yml`
- Check file paths use `.html` extension in links
- Ensure files have proper front matter

### Syntax Highlighting Not Working

- Check that code blocks use triple backticks with language identifier
- Verify Rouge is enabled in `_config.yml`
- Try rebuilding the site

### Theme Not Applying

- Verify theme name in `_config.yml`
- Check that theme is supported by GitHub Pages
- Try clearing browser cache

## Updating Documentation

### Workflow

1. Edit `.md` files in `docs/` directory
2. Test locally with Jekyll (optional)
3. Commit and push to GitHub
4. GitHub automatically rebuilds site (1-2 minutes)
5. Verify changes at your GitHub Pages URL

### Best Practices

- Use descriptive commit messages
- Test links before committing
- Keep navigation structure up to date
- Add new pages to `_config.yml` navigation
- Use relative links: `[Link](DESIGN.html)` not absolute URLs

## Custom Domain (Optional)

### Setup

1. Add `docs/CNAME` file with your domain:
   ```
   docs.aether-lang.org
   ```

2. Configure DNS records:
   ```
   Type: CNAME
   Name: docs
   Value: yourusername.github.io
   ```

3. Enable HTTPS in GitHub Pages settings

### Verification

- Wait for DNS propagation (up to 24 hours)
- Visit your custom domain
- Verify HTTPS certificate is active

## Resources

- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Jekyll Documentation](https://jekyllrb.com/docs/)
- [Markdown Guide](https://www.markdownguide.org/)
- [Cayman Theme](https://github.com/pages-themes/cayman)

## Support

For issues or questions:
- Check [GitHub Pages Status](https://www.githubstatus.com/)
- Review [Jekyll Troubleshooting](https://jekyllrb.com/docs/troubleshooting/)
- Open an issue in the Aether repository

---

**Last Updated**: 2026-04-28  
**Status**: Ready for deployment
