# Internationalization (i18n) Implementation Process

This document defines the mandatory process for implementing internationalization in all components and pages within this project.

## 🎯 Core Principle

**Every user-facing string MUST be translatable from day one.**

No hardcoded English text is permitted in any component or page. This ensures consistent international user experience and prevents technical debt.

## 📋 Mandatory Implementation Checklist

### For Every New Component

- [ ] **Import Translation Function**: `import { _ } from 'svelte-i18n';`
- [ ] **Replace All Hardcoded Text**: Use `$_('key')` for all user-visible strings
- [ ] **Add Translation Keys**: Add keys to ALL locale files (`en-US.json`, `es-ES.json`, `zh-CN.json`, `ar-SA.json`)
- [ ] **Test Language Switching**: Verify component displays correctly in all languages
- [ ] **Accessibility Text**: Translate ARIA labels, alt text, and screen reader content

### For Every New Page

- [ ] **Import Translation Function**: `import { _ } from 'svelte-i18n';`
- [ ] **Page Title**: `<title>{$_('page.pageTitle')}</title>`
- [ ] **Meta Description**: `<meta name="description" content={$_('page.pageDescription')} />`
- [ ] **All Content**: Convert all user-facing text to translation keys
- [ ] **Form Labels**: All input labels, placeholders, and validation messages
- [ ] **Error Messages**: All error states and loading messages
- [ ] **Navigation Text**: Any page-specific navigation or breadcrumbs
- [ ] **Write E2E Test**: Create Playwright test for language switching on this page

### For Every New Route

- [ ] **Follow Page Requirements**: All page implementation requirements above
- [ ] **Add Route to i18n Tests**: Include route in comprehensive i18n validation
- [ ] **Test RTL Layout**: Verify Arabic language displays correctly
- [ ] **Meta Tag Validation**: Ensure proper SEO in all languages

## 🔧 Implementation Steps

### Step 1: Planning Phase

Before writing any code:

1. **Identify All Text**: List every user-visible string
2. **Design Translation Keys**: Plan hierarchical key structure
3. **Consider Context**: Group related translations logically
4. **Plan Pluralization**: Identify text that needs plural forms

### Step 2: Translation Key Creation

Add keys to ALL locale files simultaneously:

```bash
# Add to all 4 files at once:
client/src/lib/i18n/locales/en-US.json
client/src/lib/i18n/locales/es-ES.json
client/src/lib/i18n/locales/zh-CN.json
client/src/lib/i18n/locales/ar-SA.json
```

### Step 3: Implementation

```svelte
<script lang="ts">
  import { _ } from 'svelte-i18n';
  // Component logic
</script>

<svelte:head>
  <title>{$_('component.pageTitle')}</title>
  <meta name="description" content={$_('component.pageDescription')} />
</svelte:head>

<!-- All text uses translation keys -->
<h1>{$_('component.title')}</h1>
<p>{$_('component.description')}</p>
```

### Step 4: Testing

```bash
# Run i18n validation
bun test i18n-validation

# Run E2E language switching tests
bun playwright test language-switching

# Manual testing in all languages
# 1. Switch to each language
# 2. Verify all text displays correctly
# 3. Check RTL layout for Arabic
```

## 🚨 Enforcement Rules

### Pre-commit Hooks

The following checks are enforced automatically:

1. **Translation Import Check**: Verify `import { _ } from 'svelte-i18n';` in components with user-facing text
2. **Hardcoded Text Detection**: Scan for English strings outside translation functions
3. **Translation Key Validation**: Ensure all used keys exist in all locale files
4. **Page Title Validation**: Verify all pages use translation keys for titles

### Code Review Requirements

Before any PR can be merged:

- [ ] **No Hardcoded English**: All strings use translation keys
- [ ] **Complete Translation Coverage**: Keys exist in all 4 locale files
- [ ] **E2E Tests Pass**: Language switching tests must pass
- [ ] **RTL Compatibility**: Arabic language displays correctly

### CI/CD Pipeline

Automated checks in GitHub Actions:

```yaml
- name: Validate i18n Implementation
  run: |
    bun test i18n-validation
    bun playwright test language-switching
    bun run i18n:check-coverage
```

## 📐 Translation Key Standards

### Naming Convention

```
category.subcategory.element
```

### Required Categories

| Category | Purpose | Example Keys |
|----------|---------|--------------|
| `common.*` | Shared UI elements | `common.save`, `common.cancel` |
| `nav.*` | Navigation elements | `nav.home`, `nav.profile` |
| `auth.*` | Authentication flows | `auth.login.title`, `auth.register.submit` |
| `validation.*` | Form validation | `validation.required`, `validation.email` |
| `error.*` | Error messages | `error.notFound`, `error.serverError` |
| `accessibility.*` | Screen reader text | `accessibility.skipToMain` |

### Page-Specific Requirements

Every page must have these standard keys:

```json
{
  "pageName.pageTitle": "SEO-optimized page title",
  "pageName.pageDescription": "Meta description for search engines",
  "pageName.title": "Main page heading (H1)",
  "pageName.description": "Page subtitle or description"
}
```

## 🧪 Testing Requirements

### Automated Tests

Every new page/component must include:

1. **Unit Tests**: Test translation key usage
2. **E2E Tests**: Language switching functionality
3. **Visual Tests**: RTL layout verification

### Manual Testing Checklist

Before marking i18n implementation complete:

- [ ] **English (en-US)**: Default language displays correctly
- [ ] **Spanish (es-ES)**: All text translated and displays properly
- [ ] **Chinese (zh-CN)**: Character encoding and layout work correctly
- [ ] **Arabic (ar-SA)**: RTL layout functions properly, text flows right-to-left
- [ ] **Language Persistence**: Selected language persists across navigation
- [ ] **Page Titles**: Browser tab shows translated titles
- [ ] **Meta Descriptions**: Search engine descriptions are translated
- [ ] **Form Validation**: Error messages display in selected language
- [ ] **Loading States**: Temporary messages use translation keys

## 🔍 Quality Assurance

### Static Analysis

Run these commands before committing:

```bash
# Check for hardcoded English strings
bun run i18n:lint

# Validate translation key coverage
bun run i18n:check-coverage

# Test all language switches
bun playwright test language-switching

# Verify RTL layout
bun playwright test rtl-layout
```

### Code Quality Metrics

Track these metrics in the project:

- **Translation Coverage**: 100% of user-facing strings must be translatable
- **Key Consistency**: All translation keys must exist in all locale files
- **Test Coverage**: Every page must have language switching E2E tests
- **RTL Compatibility**: Arabic language must render correctly

## 🚀 Deployment Checklist

Before deploying to production:

- [ ] **All Translation Keys Present**: No missing translations in any language
- [ ] **E2E Tests Pass**: Language switching works on all pages
- [ ] **RTL Layout Verified**: Arabic language displays correctly
- [ ] **Performance Check**: Translation bundle sizes are optimized
- [ ] **SEO Validation**: Meta tags work in all languages

## 📊 Monitoring and Maintenance

### Regular Reviews

Monthly i18n health checks:

1. **Translation Completeness**: Verify all keys exist in all languages
2. **New String Audit**: Check recent commits for hardcoded strings
3. **User Feedback**: Review user reports about translation quality
4. **Performance Monitoring**: Track translation bundle load times

### Update Process

When updating translations:

1. **Batch Updates**: Update all locale files simultaneously
2. **Version Control**: Use descriptive commit messages for translation changes
3. **Testing**: Run full E2E test suite after translation updates
4. **Documentation**: Update this process document as needed

## 🔧 Tools and Scripts

### Available Scripts

```bash
# Validate all translations
bun run i18n:validate

# Check for missing translations
bun run i18n:check-missing

# Extract hardcoded strings (for migration)
bun run i18n:extract-strings

# Generate translation coverage report
bun run i18n:coverage-report
```

### IDE Configuration

Recommended VS Code extensions:

- **i18n Ally**: Visualize and edit translation keys
- **Svelte for VS Code**: Syntax highlighting for translation functions
- **Translation Helper**: Automate translation key generation

## 📝 Documentation

### Required Documentation

For every i18n implementation:

1. **Component Documentation**: List all translation keys used
2. **API Documentation**: Document any i18n-related props or methods
3. **Testing Documentation**: Describe language switching test scenarios
4. **Maintenance Notes**: Document any special i18n considerations

### Templates

Use these templates for consistent documentation:

- [Component i18n Template](../templates/component-i18n.md)
- [Page i18n Template](../templates/page-i18n.md)
- [E2E Test Template](../templates/e2e-i18n-test.md)

## 🎯 Success Criteria

An i18n implementation is considered complete when:

- ✅ **Zero Hardcoded Strings**: No English text outside translation functions
- ✅ **Complete Coverage**: All translation keys exist in all 4 locale files
- ✅ **Tests Pass**: All automated i18n tests pass
- ✅ **Manual Verification**: All languages display correctly in browser
- ✅ **RTL Support**: Arabic language renders with proper right-to-left layout
- ✅ **Performance**: Translation bundles load efficiently
- ✅ **Accessibility**: Screen reader text is translated
- ✅ **SEO**: Meta tags work in all languages

## 📞 Support and Resources

### Getting Help

1. **Review Documentation**: Check this process document and the README
2. **Study Examples**: Look at existing translated components
3. **Run Validation**: Use automated tools to identify issues
4. **Ask for Review**: Request i18n-specific code review

### Key Files

- `client/src/lib/i18n/README.md` - Implementation guide
- `client/src/lib/i18n/index.ts` - Main i18n configuration
- `client/e2e/language-switching.test.ts` - E2E test examples
- `documentation/I18N_PROCESS.md` - This process document

## 🔄 Process Updates

This process document is a living document. Updates should be made when:

- New languages are added to the system
- New tools or testing methods are introduced
- Process improvements are identified
- User feedback suggests changes

All process updates require:
- Team review and approval
- Documentation of the change rationale
- Update of related templates and examples
- Communication to all developers

---

**Remember: i18n is not optional - it's a core requirement for every user-facing element in this application.**
