// server/load-tests/chat-load.js
// k6 load test for AI chat endpoints
//
// Run with: k6 run --config config/load.json chat-load.js
// Or: k6 run --vus 5 --duration 60s chat-load.js
//
// Note: This test requires authenticated users. It will register/login
// users in the setup phase and use their tokens for chat requests.

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { SharedArray } from 'k6/data';

// Custom metrics
const errorRate = new Rate('errors');
const chatDuration = new Trend('chat_duration', true);
const chatContextualDuration = new Trend('chat_contextual_duration', true);

// Test configuration - can be overridden by config files
export const options = {
  thresholds: {
    // P95 response time for chat must be under 2000ms
    'http_req_duration{endpoint:chat}': ['p(95)<2000'],
    // P95 response time for contextual chat must be under 2500ms
    'http_req_duration{endpoint:chat_contextual}': ['p(95)<2500'],
    // Overall error rate must be under 0.1%
    errors: ['rate<0.001'],
    // Custom chat duration threshold
    chat_duration: ['p(95)<2000'],
  },
};

// Environment configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8081';

// Sample chat prompts for realistic load testing
const chatPrompts = [
  'Hello, how are you?',
  'What is the capital of France?',
  'Explain quantum computing in simple terms.',
  'Write a haiku about programming.',
  'What are the benefits of test-driven development?',
  'How do I handle errors in Rust?',
  'Describe the SOLID principles.',
  'What is the difference between REST and GraphQL?',
];

// Get a random prompt
function getRandomPrompt() {
  return chatPrompts[Math.floor(Math.random() * chatPrompts.length)];
}

// Generate unique test user for setup
function generateTestUser(index) {
  const timestamp = Date.now();
  return {
    email: `chattest_${index}_${timestamp}@test.example.com`,
    password: 'ChatTest_SecurePass_123!',
  };
}

// Register and login a user, returning the auth token
function setupTestUser(index) {
  const user = generateTestUser(index);

  // Register
  const registerRes = http.post(
    `${BASE_URL}/api/auth/register`,
    JSON.stringify({ email: user.email, password: user.password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (registerRes.status !== 201) {
    console.error(`Setup: Register failed for user ${index}: ${registerRes.status}`);
    return null;
  }

  // Login to get fresh token
  const loginRes = http.post(
    `${BASE_URL}/api/auth/login`,
    JSON.stringify({ email: user.email, password: user.password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (loginRes.status !== 200) {
    console.error(`Setup: Login failed for user ${index}: ${loginRes.status}`);
    return null;
  }

  try {
    const body = JSON.parse(loginRes.body);
    return body.auth_token;
  } catch (e) {
    console.error(`Setup: Failed to parse login response for user ${index}`);
    return null;
  }
}

// Store tokens from setup
let authTokens = [];

// Test scenarios
export default function () {
  // Get token for this VU (cycle through available tokens)
  const tokenIndex = (__VU - 1) % authTokens.length;
  const token = authTokens[tokenIndex];

  if (!token) {
    console.error(`No valid token available for VU ${__VU}`);
    errorRate.add(true);
    sleep(1);
    return;
  }

  const headers = {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${token}`,
  };

  // Scenario 1: Basic chat request
  const chatPayload = JSON.stringify({
    message: getRandomPrompt(),
    model: __ENV.AI_MODEL || 'gpt-3.5-turbo',
  });

  const chatParams = {
    headers: headers,
    tags: { endpoint: 'chat' },
    timeout: '30s', // AI requests can take longer
  };

  const chatStart = Date.now();
  const chatRes = http.post(
    `${BASE_URL}/api/ai/chat`,
    chatPayload,
    chatParams
  );
  chatDuration.add(Date.now() - chatStart);

  const chatSuccess = check(chatRes, {
    'chat: status is 200': (r) => r.status === 200,
    'chat: has response': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.response !== undefined || body.message !== undefined || body.choices !== undefined;
      } catch {
        return false;
      }
    },
    'chat: response time under 2s': (r) => r.timings.duration < 2000,
  });

  errorRate.add(!chatSuccess);

  if (!chatSuccess && chatRes.status !== 200) {
    console.error(`Chat failed: ${chatRes.status} - ${chatRes.body?.substring(0, 200)}`);
  }

  sleep(2); // Think time - AI chat has natural pauses

  // Scenario 2: Contextual chat request (with conversation history)
  // Only run this 30% of the time to mix up request types
  if (Math.random() < 0.3) {
    const contextualPayload = JSON.stringify({
      message: getRandomPrompt(),
      model: __ENV.AI_MODEL || 'gpt-3.5-turbo',
      context: [
        { role: 'user', content: 'Hello!' },
        { role: 'assistant', content: 'Hello! How can I help you today?' },
      ],
    });

    const contextualParams = {
      headers: headers,
      tags: { endpoint: 'chat_contextual' },
      timeout: '30s',
    };

    const contextualStart = Date.now();
    const contextualRes = http.post(
      `${BASE_URL}/api/ai/chat/contextual`,
      contextualPayload,
      contextualParams
    );
    chatContextualDuration.add(Date.now() - contextualStart);

    const contextualSuccess = check(contextualRes, {
      'contextual_chat: status is 200': (r) => r.status === 200,
      'contextual_chat: has response': (r) => {
        try {
          const body = JSON.parse(r.body);
          return body.response !== undefined || body.message !== undefined || body.choices !== undefined;
        } catch {
          return false;
        }
      },
    });

    errorRate.add(!contextualSuccess);

    if (!contextualSuccess && contextualRes.status !== 200) {
      console.error(`Contextual chat failed: ${contextualRes.status}`);
    }

    sleep(1);
  }

  sleep(1); // Additional think time
}

// Lifecycle hooks
export function setup() {
  console.log(`Starting chat load test against ${BASE_URL}`);

  // Verify server is reachable
  const healthRes = http.get(`${BASE_URL}/api/ai/health`);
  if (healthRes.status !== 200) {
    throw new Error(`Server health check failed: ${healthRes.status}`);
  }
  console.log('Server health check passed');

  // Create test users and get their tokens
  // Create more users than VUs to ensure availability
  const numUsers = Math.max(10, (__ENV.K6_VUS || 5) * 2);
  const tokens = [];

  console.log(`Setting up ${numUsers} test users...`);
  for (let i = 0; i < numUsers; i++) {
    const token = setupTestUser(i);
    if (token) {
      tokens.push(token);
    }
    // Small delay to avoid overwhelming the server during setup
    sleep(0.1);
  }

  if (tokens.length === 0) {
    throw new Error('Failed to create any test users');
  }

  console.log(`Successfully created ${tokens.length} test users`);

  // Store tokens globally for use in default function
  authTokens = tokens;

  return { startTime: Date.now(), userCount: tokens.length };
}

export function teardown(data) {
  const duration = (Date.now() - data.startTime) / 1000;
  console.log(`Chat load test completed in ${duration.toFixed(2)}s`);
  console.log(`Used ${data.userCount} test users`);
}
