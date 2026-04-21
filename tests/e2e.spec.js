import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:3000';

test.describe('ailib-wasm-test e2e', () => {
  test.beforeAll(async () => {
    // Verify server is up
    const resp = await fetch(`${BASE_URL}/health`);
    expect(resp.ok).toBeTruthy();
  });

  test('health endpoint returns ok', async ({ request }) => {
    const resp = await request.get(`${BASE_URL}/health`);
    expect(resp.ok).toBeTruthy();
    const body = await resp.json();
    expect(body.status).toBe('ok');
    expect(body.version).toBeDefined();
  });

  test('static index.html loads', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await page.waitForLoadState('networkidle');
    const title = await page.title();
    expect(title).toContain('ailib-wasm-test');
  });

  test('wasm module loads successfully', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    // Wait for wasm status to show "ready"
    const statusEl = page.locator('#wasm-status');
    await expect(statusEl).toHaveClass(/ready/, { timeout: 10000 });
    await expect(statusEl).toHaveText('WASM ready');
  });

  test('chat with DeepSeek (streaming)', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await expect(page.locator('#wasm-status')).toHaveClass(/ready/, { timeout: 10000 });

    // Select DeepSeek
    await page.selectOption('#model-select', 'deepseek/deepseek-chat');

    // Type and send message
    await page.fill('#message-input', 'Say exactly: hello world');
    await page.click('#send-btn');

    // Wait for assistant message to appear
    const assistantMsg = page.locator('.message.assistant').first();
    await expect(assistantMsg).toBeVisible({ timeout: 10000 });

    // Wait for streaming to finish (send button reverts to "Send")
    await expect(page.locator('#send-btn')).toHaveText('Send', { timeout: 90000 });

    // Verify content contains some text
    const content = await assistantMsg.locator('.message-content').textContent();
    expect(content).toBeTruthy();
    expect(content.length).toBeGreaterThan(0);

    // Verify wasm badge
    const badge = await assistantMsg.locator('.wasm-badge').textContent();
    expect(badge).toContain('wasm');
  });

  test('chat with Groq (streaming)', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await expect(page.locator('#wasm-status')).toHaveClass(/ready/, { timeout: 10000 });

    await page.selectOption('#model-select', { label: 'Groq • Llama 3.1 8B' });

    await page.fill('#message-input', 'Say exactly: greetings from groq');
    await page.click('#send-btn');

    const assistantMsg = page.locator('.message.assistant').first();
    await expect(assistantMsg).toBeVisible({ timeout: 10000 });

    await expect(page.locator('#send-btn')).toHaveText('Send', { timeout: 90000 });

    const content = await assistantMsg.locator('.message-content').textContent();
    expect(content).toBeTruthy();
    expect(content.length).toBeGreaterThan(0);
  });

  test('new chat clears history', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await expect(page.locator('#wasm-status')).toHaveClass(/ready/, { timeout: 10000 });

    await page.fill('#message-input', 'Hello');
    await page.click('#send-btn');
    const assistantMsg = page.locator('.message.assistant').first();
    await expect(assistantMsg).toBeVisible({ timeout: 10000 });
    await expect(page.locator('#send-btn')).toHaveText('Send', { timeout: 90000 });

    await page.click('#new-chat-btn');

    const messages = await page.locator('.message.user').count();
    expect(messages).toBe(0);
    const assistantMessages = await page.locator('.message.assistant').count();
    expect(assistantMessages).toBe(0);
  });

  test('wasm functions are callable from JS', async ({ page }) => {
    await page.goto(`${BASE_URL}/`);
    await expect(page.locator('#wasm-status')).toHaveClass(/ready/, { timeout: 10000 });

    const result = await page.evaluate(async () => {
      const wasm = await import('./wasm/ailib_wasm.js');
      await wasm.default('./wasm/ailib_wasm_bg.wasm');

      const messages = JSON.stringify([{ role: 'user', content: 'test' }]);
      const buildResult = wasm.build_chat_request(messages, 'test-model', 0.7, 1024, true);
      const body = buildResult.body();
      const stream = buildResult.stream();
      buildResult.free();

      const isDone = wasm.is_stream_done('[DONE]');
      const isNotDone = wasm.is_stream_done('{"choices":[]}');

  const errClass = wasm.classify_error(429);
  const errCode = errClass.code();
  const errRetryable = errClass.retryable();
  errClass.free();

  const capJson = wasm.ailib_invoke(JSON.stringify({ op: 'capabilities', version: 1 }));
  const caps = JSON.parse(capJson);
  const abiJson = wasm.ailib_invoke(JSON.stringify({ op: 'abi_version' }));

  return {
    bodyExists: !!body,
    streamValue: stream,
    isDoneOnDone: isDone,
    isDoneOnData: isNotDone,
    errCode,
    errRetryable,
    capVersion: caps.version,
    abiInvoke: JSON.parse(abiJson),
  };
});

  expect(result.bodyExists).toBeTruthy();
  expect(result.streamValue).toBeTruthy();
  expect(result.isDoneOnDone).toBeTruthy();
  expect(result.isDoneOnData).toBeFalsy();
  expect(result.errCode).toBe(429);
  expect(result.errRetryable).toBeTruthy();
  expect(result.capVersion).toBe(1);
  expect(result.abiInvoke.version).toBe(1);
  });
});
