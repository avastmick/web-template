# Web Template Usage Guide

This guide explains how to use the `create-web-template` CLI tool to scaffold new projects from the web-template.

## Table of Contents

1. [Installation](#installation)
2. [Creating a New Project](#creating-a-new-project)
3. [Project Configuration](#project-configuration)
4. [Features](#features)
5. [Updating Projects](#updating-projects)
6. [Template Development](#template-development)

## Installation

### From Source

1. Clone the web-template repository:
   ```bash
   git clone https://github.com/your-org/web-template.git
   cd web-template
   ```

2. Build the CLI tool:
   ```bash
   just template-build
   ```

3. The compiled binary will be at `scripts/create-web-template/target/release/create-web-template`

4. (Optional) Add to your PATH for easy access:
   ```bash
   export PATH="$PATH:/path/to/web-template/scripts/create-web-template/target/release"
   ```

## Creating a New Project

### Interactive Mode (Recommended)

Run the CLI tool without the `--no-interactive` flag:

```bash
create-web-template new my-awesome-project
```

The interactive wizard will guide you through:
- Project name validation
- Project directory selection
- Description and author information
- Feature selection
- Database configuration
- OAuth provider setup (if selected)
- Payment integration setup (if selected)

### Non-Interactive Mode

For automated setups or CI/CD pipelines:

```bash
create-web-template new my-project \
  --path ./projects/my-project \
  --no-interactive \
  --features local_auth,stripe_payment,chat
```

### Command Options

- `--path <PATH>`: Target directory (defaults to project name in current directory)
- `--no-interactive`: Skip interactive setup wizard
- `--features <FEATURES>`: Comma-separated list of features to enable
- `--template <PATH>`: Path to template source (defaults to current template)
- `--dry-run`: Preview changes without creating files
- `--verbose`: Enable verbose output

## Project Configuration

### Available Features

| Feature | Description | Default |
|---------|-------------|---------|
| `local_auth` | Email/password authentication | ✓ |
| `google_auth` | Google OAuth authentication | ✓ |
| `github_auth` | GitHub OAuth authentication | ✓ |
| `stripe_payment` | Stripe payment integration | ✓ |
| `chat` | Chat functionality | ✓ |

### Database Options

- **SQLite** (default): Perfect for development and small deployments
- **PostgreSQL**: For production deployments requiring advanced features

### Environment Variables

After project creation, configure your `.envrc` file with:

```bash
# Core Configuration
export DATABASE_URL="sqlite://data/app.db"
export JWT_SECRET="your-secret-key"

# OAuth (if enabled)
export GOOGLE_CLIENT_ID="your-google-client-id"
export GOOGLE_CLIENT_SECRET="your-google-client-secret"
export GITHUB_CLIENT_ID="your-github-client-id"
export GITHUB_CLIENT_SECRET="your-github-client-secret"

# Stripe (if enabled)
export STRIPE_SECRET_KEY="your-stripe-secret-key"
export STRIPE_WEBHOOK_SECRET="your-stripe-webhook-secret"
```

## Updating Projects

The template includes an update mechanism to pull in the latest improvements:

### Check for Updates

```bash
create-web-template update --dry-run
```

### Update Project

```bash
# Update all template files
create-web-template update

# Force update (overwrites local changes)
create-web-template update --force

# Update specific files only
create-web-template update --only "*.toml,*.json"

# Exclude files from update
create-web-template update --exclude "*.md"
```

### How Updates Work

1. The tool compares your project's template version with the latest
2. Shows files that will be updated
3. Preserves your custom code while updating template files
4. Updates the project metadata (`.web-template.json`)

**Note**: Always commit your changes before updating and review the changes afterward.

## Template Development

### Testing Template Changes

Use the provided just commands:

```bash
# Test template generation
just template-test my-test-project

# Clean up test projects
just template-clean

# Run template checks
just check-template
```

### Template Configuration

The `template.config.json` file controls:
- Available features and their dependencies
- File mappings and transformations
- Variable definitions
- Post-processing steps

### Variable Substitution

The template supports these variables:
- `{{project_name}}`: The project name
- `{{project_description}}`: Project description
- `{{author_name}}`: Author name
- `{{author_email}}`: Author email

### Adding New Features

1. Update `template.config.json` with the new feature
2. Add feature-specific files to the template
3. Update the wizard to handle feature configuration
4. Test the feature with `just template-test`

## Common Workflows

### Quick Start

```bash
# Create project
create-web-template new my-app

# Navigate to project
cd my-app

# Set up environment
direnv allow

# Install dependencies and start development
just setup
just dev
```

### Production Deployment

```bash
# Build for production
just build

# Run production build
just prod
```

### Running Tests

```bash
# Run all tests
just test

# Run specific tests
just test-server auth
just test-client payment
just test-e2e login
```

## Troubleshooting

### Common Issues

1. **"Not a valid web-template project directory"**
   - Ensure you're running the update command from a project created with this template
   - Check that `justfile` exists in the project root

2. **"Could not find template root"**
   - Run the command from within the web-template directory
   - Or specify `--template /path/to/web-template`

3. **Feature conflicts**
   - Some features may conflict (e.g., different database backends)
   - The wizard will validate feature compatibility

4. **Build failures after update**
   - Run `just clean && just setup` to refresh dependencies
   - Check for breaking changes in the update notes

### Getting Help

- Check the project README for setup instructions
- Review `CURRENT_TASKS.md` for development status
- Submit issues to the project repository

## Best Practices

1. **Version Control**: Always initialize git and commit regularly
2. **Environment Variables**: Never commit `.envrc`, use `.envrc.example` as reference
3. **Updates**: Test updates in a separate branch first
4. **Features**: Only enable features you need to reduce complexity
5. **Security**: Regularly update dependencies with `cargo update` and `bun update`

## Next Steps

After creating your project:

1. Review the generated README.md for project-specific instructions
2. Configure your environment variables in `.envrc`
3. Set up your database with `just db-setup`
4. Run tests to ensure everything works: `just test`
5. Start development with `just dev`

Happy coding! 🚀
