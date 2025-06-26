# Project Requirements Documentation

Out of the box needs:

- Beautiful web application with easy CSS-only changes, including:
    - Dark/Light modes
    - Configurable colour-schemes
- Fast server with modular components that are easy to add, including:
    - Database access
    - Server-side components
    - Generative AI integration
    - Easy AI integration into any provider
- Highly secure user registration, authentication, and profile management
- Initial auth providers:
    - Local using username and password, where username is valid, unique email
    - Google via OAuth
- Payment integration using Stripe
- Deployment to:
    - GCP Cloud Run
    - Vercel
    - Supabase

## Authentication and Payment Flow

### Navigation and Route Structure

1. **Landing Page (`/`)**:
   - The base route serves as a redirect dispatcher
   - If user is unauthenticated: Redirect to `/login`
   - If user is authenticated and has access (invite or payment): Redirect to `/chat`
   - No UI is displayed on this route - it only performs redirects

2. **Registration Page (`/register`)**:
   - Dedicated page for user registration
   - Shows ONLY the registration form
   - Email/password registration or OAuth options
   - Successful registration: Redirect to `/login` with success message
   - Failed registration: Stay on `/register` with error message
   - EXCEPTION: OAuth users who successfully register are automatically logged in and follow the login flow

3. **Login Page (`/login`)**:
   - Dedicated page for user authentication
   - Shows ONLY the login form
   - Displays success message if redirected from registration
   - Upon successful login:
     - Check if user has invite or active payment
     - If YES: Redirect to `/` (which redirects to `/chat`)
     - If NO: Redirect to `/payment`

4. **Payment Page (`/payment`)**:
   - Requires authentication (redirect to `/login` if not authenticated)
   - Shows ONLY the Stripe payment form
   - Uses Stripe Web Elements (prebuilt components)
   - Reference: https://docs.stripe.com/payments/elements
   - Successful payment: Redirect to `/` (which redirects to `/chat`)
   - Failed/cancelled payment: Stay on `/payment` with appropriate message

5. **Chat Page (`/chat`)**:
   - The main application interface
   - Requires authentication AND access (invite or payment)
   - If not authenticated: Redirect to `/login`
   - If authenticated but no access: Redirect to `/payment`

### Authorization Rules

All routes enforce authorization:
- **Unauthenticated users**: Can only access `/login` and `/register`
- **Authenticated users without access**: Can access `/payment`
- **Authenticated users with access**: Can access `/chat`

Unauthorized access attempts result in redirects:
- Not registered → `/register`
- Not logged in → `/login`
- Not paid/invited → `/payment`

### UI Route Isolation

**CRITICAL**: Each route must display ONLY its designated UI elements:
- `/register` - ONLY registration form and related elements
- `/login` - ONLY login form and related elements
- `/payment` - ONLY payment form and related elements
- `/chat` - ONLY chat interface
- `/` - NO UI, only redirect logic

No route should ever display UI elements from other routes. The application must enforce strict route isolation to prevent UI context bleeding.
