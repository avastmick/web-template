# AI Module Architecture

This module contains all AI-related functionality for the web-template server, organized for clarity, extensibility, and maintainability.

## Directory Structure

```
ai/
├── providers/        # AI provider implementations (OpenRouter, etc.)
├── services/         # Business logic services using AI
├── prompts/         # Prompt templates and management
├── models/          # AI-specific data structures
├── config/          # AI configuration and settings
└── error.rs         # AI-specific error types
```

## Module Organization

### Providers (`providers/`)
- **Purpose**: Abstracts the AI provider interface
- **Key Components**:
  - `traits.rs`: Defines the `AiProvider` trait
  - `openrouter.rs`: OpenRouter implementation using OpenAI SDK
- **Configuration**:
  - `AI_PROVIDER_ENDPOINT`: Override the API endpoint (defaults to OpenRouter)
  - Supports any OpenAI-compatible API endpoint

### Services (`services/`)
- **Purpose**: Business logic that uses AI capabilities
- **Key Components**:
  - `orchestrator.rs`: Main service that coordinates AI interactions
  - `analyst.rs`: Business analyst persona for issue creation
  - `duplicate_detector.rs`: Semantic duplicate detection
  - `issue_sizer.rs`: Automatic issue complexity estimation

### Prompts (`prompts/`)
- **Purpose**: Centralized prompt management
- **Key Components**:
  - `templates/`: TOML files containing prompt templates
  - `manager.rs`: Loads and manages prompt templates
- **Benefits**:
  - Version controlled prompts
  - Easy to modify without code changes
  - Support for prompt variants and A/B testing

### Models (`models/`)
- **Purpose**: AI-specific data structures
- **Key Components**:
  - `chat.rs`: Chat messages, requests, responses
  - `analysis.rs`: Issue analysis results
  - `usage.rs`: Token usage tracking

### Config (`config/`)
- **Purpose**: AI configuration management
- **Key Components**:
  - `settings.rs`: Model selection, temperature, max tokens, etc.
- **Environment Variables**:
  - `OPENROUTER_API_KEY`: API key for OpenRouter
  - `AI_DEFAULT_MODEL`: Default model to use (e.g., "anthropic/claude-3.5-sonnet")
  - `AI_MAX_TOKENS`: Maximum tokens per request
  - `AI_TEMPERATURE`: Model temperature setting

## Usage Examples

### Using the AI Orchestrator

```rust
use crate::ai::services::AiOrchestrator;

// In a handler
let orchestrator = AiOrchestrator::new(ai_provider, prompt_manager);
let analysis = orchestrator.analyze_issue(issue_description).await?;
```

### Adding a New AI Provider

1. Create `providers/new_provider.rs`
2. Implement the `AiProvider` trait
3. Add to `providers/mod.rs`
4. Update configuration to support provider selection

### Adding a New Prompt Template

1. Create a TOML file in `prompts/templates/`
2. Define the prompt with placeholders:
   ```toml
   name = "business_analyst"
   description = "Acts as a business analyst to help create comprehensive issues"

   [variables]
   issue_type = "The type of issue (feature, bug, etc.)"
   description = "The initial issue description"

   [prompt]
   system = """
   You are an experienced business analyst helping to create well-defined issues.
   Your goal is to ensure all issues have clear acceptance criteria and technical considerations.
   """

   user = """
   Help me create a {issue_type} issue:
   {description}
   """
   ```

## Best Practices

1. **Provider Agnostic**: Services should depend on the `AiProvider` trait, not specific implementations
2. **Prompt Versioning**: Include version numbers in prompt templates for A/B testing
3. **Error Handling**: Use specific error types for different failure modes
4. **Token Limits**: Always respect model token limits and implement chunking if needed
5. **Caching**: Consider caching responses for duplicate detection to reduce API calls
6. **Monitoring**: Log all AI interactions for debugging and cost tracking

## Testing

- Unit tests for each service with mocked providers
- Integration tests using a test provider implementation
- Prompt validation tests to ensure all variables are defined
- Performance tests for duplicate detection with large datasets
