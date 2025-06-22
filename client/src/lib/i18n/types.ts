// Type definitions for translation keys
// This provides type safety for translation keys throughout the application

export interface TranslationSchema {
	// Common
	'common.loading': string;
	'common.error': string;
	'common.success': string;
	'common.cancel': string;
	'common.save': string;
	'common.delete': string;
	'common.edit': string;
	'common.back': string;
	'common.next': string;
	'common.submit': string;
	'common.confirm': string;
	'common.close': string;

	// Navigation
	'nav.home': string;
	'nav.dashboard': string;
	'nav.profile': string;
	'nav.settings': string;
	'nav.logout': string;

	// Authentication
	'auth.login.title': string;
	'auth.login.subtitle': string;
	'auth.login.email': string;
	'auth.login.password': string;
	'auth.login.submit': string;
	'auth.login.forgotPassword': string;
	'auth.login.noAccount': string;
	'auth.login.signUp': string;
	'auth.login.error': string;
	'auth.login.success': string;
	'auth.login.or': string;
	'auth.login.withGoogle': string;
	'auth.login.withGithub': string;

	'auth.register.title': string;
	'auth.register.subtitle': string;
	'auth.register.email': string;
	'auth.register.password': string;
	'auth.register.confirmPassword': string;
	'auth.register.submit': string;
	'auth.register.hasAccount': string;
	'auth.register.signIn': string;
	'auth.register.error': string;
	'auth.register.success': string;
	'auth.register.passwordMismatch': string;
	'auth.register.inviteRequired': string;

	'auth.logout.message': string;
	'auth.logout.confirm': string;

	'auth.forgotPassword.title': string;
	'auth.forgotPassword.subtitle': string;
	'auth.forgotPassword.email': string;
	'auth.forgotPassword.submit': string;
	'auth.forgotPassword.backToLogin': string;
	'auth.forgotPassword.success': string;
	'auth.forgotPassword.error': string;

	// Profile
	'profile.title': string;
	'profile.email': string;
	'profile.name': string;
	'profile.provider': string;
	'profile.joinedDate': string;
	'profile.updateSuccess': string;
	'profile.updateError': string;

	// Settings
	'settings.title': string;
	'settings.theme.title': string;
	'settings.theme.light': string;
	'settings.theme.dark': string;
	'settings.theme.system': string;
	'settings.language.title': string;
	'settings.language.select': string;

	// Dashboard
	'dashboard.title': string;
	'dashboard.welcome': string;
	'dashboard.statistics': string;

	// Errors
	'error.404.title': string;
	'error.404.message': string;
	'error.500.title': string;
	'error.500.message': string;
	'error.generic': string;
	'error.network': string;
	'error.unauthorized': string;

	// Validation
	'validation.required': string;
	'validation.email': string;
	'validation.minLength': string;
	'validation.maxLength': string;
	'validation.passwordStrength': string;
	'validation.confirmPassword': string;
	'validation.passwordLowercase': string;
	'validation.passwordUppercase': string;
	'validation.passwordNumber': string;

	// Home
	'home.title': string;
	'home.description': string;
	'home.welcomeBack': string;
	'home.viewProfile': string;
	'home.createAccount': string;
	'home.features': string;
	'home.feature.auth.title': string;
	'home.feature.auth.description': string;
	'home.feature.performance.title': string;
	'home.feature.performance.description': string;
	'home.feature.theming.title': string;
	'home.feature.theming.description': string;
	'home.feature.types.title': string;
	'home.feature.types.description': string;
}

// Type for translation function with parameters
export type TranslationFunction = (
	key: keyof TranslationSchema,
	params?: Record<string, string | number>
) => string;

// Export all translation keys as a const array for validation
export const TRANSLATION_KEYS = [
	// Common
	'common.loading',
	'common.error',
	'common.success',
	'common.cancel',
	'common.save',
	'common.delete',
	'common.edit',
	'common.back',
	'common.next',
	'common.submit',
	'common.confirm',
	'common.close',

	// Navigation
	'nav.home',
	'nav.dashboard',
	'nav.profile',
	'nav.settings',
	'nav.logout',

	// Authentication
	'auth.login.title',
	'auth.login.subtitle',
	'auth.login.email',
	'auth.login.password',
	'auth.login.submit',
	'auth.login.forgotPassword',
	'auth.login.noAccount',
	'auth.login.signUp',
	'auth.login.error',
	'auth.login.success',
	'auth.login.or',
	'auth.login.withGoogle',
	'auth.login.withGithub',

	'auth.register.title',
	'auth.register.subtitle',
	'auth.register.email',
	'auth.register.password',
	'auth.register.confirmPassword',
	'auth.register.submit',
	'auth.register.hasAccount',
	'auth.register.signIn',
	'auth.register.error',
	'auth.register.success',
	'auth.register.passwordMismatch',
	'auth.register.inviteRequired',

	'auth.logout.message',
	'auth.logout.confirm',

	'auth.forgotPassword.title',
	'auth.forgotPassword.subtitle',
	'auth.forgotPassword.email',
	'auth.forgotPassword.submit',
	'auth.forgotPassword.backToLogin',
	'auth.forgotPassword.success',
	'auth.forgotPassword.error',

	// Profile
	'profile.title',
	'profile.email',
	'profile.name',
	'profile.provider',
	'profile.joinedDate',
	'profile.updateSuccess',
	'profile.updateError',

	// Settings
	'settings.title',
	'settings.theme.title',
	'settings.theme.light',
	'settings.theme.dark',
	'settings.theme.system',
	'settings.language.title',
	'settings.language.select',

	// Dashboard
	'dashboard.title',
	'dashboard.welcome',
	'dashboard.statistics',

	// Errors
	'error.404.title',
	'error.404.message',
	'error.500.title',
	'error.500.message',
	'error.generic',
	'error.network',
	'error.unauthorized',

	// Validation
	'validation.required',
	'validation.email',
	'validation.minLength',
	'validation.maxLength',
	'validation.passwordStrength',
	'validation.confirmPassword',
	'validation.passwordLowercase',
	'validation.passwordUppercase',
	'validation.passwordNumber',

	// Home
	'home.title',
	'home.description',
	'home.welcomeBack',
	'home.viewProfile',
	'home.createAccount',
	'home.features',
	'home.feature.auth.title',
	'home.feature.auth.description',
	'home.feature.performance.title',
	'home.feature.performance.description',
	'home.feature.theming.title',
	'home.feature.theming.description',
	'home.feature.types.title',
	'home.feature.types.description'
] as const;

export type TranslationKey = (typeof TRANSLATION_KEYS)[number];
