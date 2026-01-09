// server/load-tests/auth-load.js
// k6 load test for authentication endpoints
//
// Run with: k6 run --config config/load.json auth-load.js
// Or: k6 run --vus 10 --duration 30s auth-load.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const loginDuration = new Trend('login_duration', true);
const registerDuration = new Trend('register_duration', true);

// Test configuration - can be overridden by config files
export const options = {
  thresholds: {
    // P95 response time for login must be under 200ms
    'http_req_duration{endpoint:login}': ['p(95)<200'],
    // P95 response time for register must be under 300ms
    'http_req_duration{endpoint:register}': ['p(95)<300'],
    // Overall error rate must be under 0.1%
    errors: ['rate<0.001'],
    // Custom login duration threshold
    login_duration: ['p(95)<200'],
  },
};

// Environment configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8081';

// Generate unique test user for each VU iteration
function generateTestUser() {
  const timestamp = Date.now();
  const vuId = __VU || 0;
  const iteration = __ITER || 0;
  return {
    email: `loadtest_${vuId}_${iteration}_${timestamp}@test.example.com`,
    password: 'LoadTest_SecurePass_123!',
  };
}

// Test scenarios
export default function () {
  const user = generateTestUser();

  // Scenario 1: Register a new user
  const registerPayload = JSON.stringify({
    email: user.email,
    password: user.password,
  });

  const registerParams = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { endpoint: 'register' },
  };

  const registerStart = Date.now();
  const registerRes = http.post(
    `${BASE_URL}/api/auth/register`,
    registerPayload,
    registerParams
  );
  registerDuration.add(Date.now() - registerStart);

  const registerSuccess = check(registerRes, {
    'register: status is 201': (r) => r.status === 201,
    'register: has auth_token': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.auth_token !== undefined;
      } catch {
        return false;
      }
    },
  });

  errorRate.add(!registerSuccess);

  if (!registerSuccess) {
    console.error(`Register failed: ${registerRes.status} - ${registerRes.body}`);
    return;
  }

  sleep(0.5); // Brief pause between register and login

  // Scenario 2: Login with the registered user
  const loginPayload = JSON.stringify({
    email: user.email,
    password: user.password,
  });

  const loginParams = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { endpoint: 'login' },
  };

  const loginStart = Date.now();
  const loginRes = http.post(
    `${BASE_URL}/api/auth/login`,
    loginPayload,
    loginParams
  );
  loginDuration.add(Date.now() - loginStart);

  const loginSuccess = check(loginRes, {
    'login: status is 200': (r) => r.status === 200,
    'login: has auth_token': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.auth_token !== undefined;
      } catch {
        return false;
      }
    },
    'login: response time OK': (r) => r.timings.duration < 200,
  });

  errorRate.add(!loginSuccess);

  if (!loginSuccess) {
    console.error(`Login failed: ${loginRes.status} - ${loginRes.body}`);
  }

  sleep(1); // Think time between iterations
}

// Lifecycle hooks
export function setup() {
  console.log(`Starting auth load test against ${BASE_URL}`);

  // Verify server is reachable
  const healthRes = http.get(`${BASE_URL}/api/ai/health`);
  if (healthRes.status !== 200) {
    throw new Error(`Server health check failed: ${healthRes.status}`);
  }

  console.log('Server health check passed');
  return { startTime: Date.now() };
}

export function teardown(data) {
  const duration = (Date.now() - data.startTime) / 1000;
  console.log(`Auth load test completed in ${duration.toFixed(2)}s`);
}
