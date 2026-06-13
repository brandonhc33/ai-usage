#!/usr/bin/env node
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { clearStaleSingletonLock } from './lib/chrome-profile.mjs';
import { withDisplay } from './lib/xvfb.mjs';

const HOME = os.homedir();
const CACHE_DIR = path.join(HOME, '.cache', 'ai-usage-status');
const CONFIG_FILE = path.join(HOME, '.config', 'ai-usage-status', 'config.json');
const OUT_FILE = path.join(CACHE_DIR, 'status.json');
const LAST_GOOD_FILE = path.join(CACHE_DIR, 'last-good-status.json');

const DEFAULT_CONFIG = {
  schema_version: 1,
  primary: { provider: 'codex', metric: 'five_hour', display_mode: 'used_percent' },
  thresholds: { stale_after_minutes: 15, last_good_valid_minutes: 30 },
  providers: {
    codex: {
      enabled: true,
      url: 'https://chatgpt.com/codex/cloud/settings/analytics#usage',
      profile_dir: '~/.config/ai-usage-status/profiles/chatgpt',
      headless: true,
      wait_ms: 8000,
      debug_text: false,
    },
    claude: {
      enabled: true,
      url: 'https://claude.ai/new#settings/usage',
      profile_dir: '~/.config/ai-usage-status/profiles/claude',
      headless: true,
      wait_ms: 8000,
      debug_text: false,
    },
  },
};

function expandHome(value) {
  return value?.replace(/^~(?=$|\/)/, HOME);
}

function normalizeText(raw) {
  return String(raw ?? '')
    .replace(/\u00a0/g, ' ')
    .replace(/[ \t]+/g, ' ')
    .replace(/\n+/g, '\n')
    .trim()
    .replace(/\n/g, ' ');
}

function n(value) {
  if (value === undefined || value === null) return null;
  const parsed = Number(String(value).replace(',', '.'));
  return Number.isFinite(parsed) ? parsed : null;
}

function parseCodex(raw) {
  const text = normalizeText(raw);

  const fiveHour = text.match(
    /(Límite de uso de 5 horas|5[\s-]?hour usage limit)\s+(\d+)\s*%\s*(restante|remaining).*?(Se reinicia a las|Resets)\s+(.+?)(?=Límite de uso semanal|Weekly usage limit|Créditos restantes|Credits remaining|Recarga automática|Auto-reload|$)/i,
  );

  const weekly = text.match(
    /(Límite de uso semanal|Weekly usage limit)\s+(\d+)\s*%\s*(restante|remaining).*?(Se reinicia a las|Resets)\s+(.+?)(?=Créditos restantes|Credits remaining|Recarga automática|Auto-reload|$)/i,
  );

  const credits = text.match(/(Créditos restantes|Credits remaining)\s+(\d+)/i);

  const fiveRemaining = fiveHour ? n(fiveHour[2]) : null;
  const weekRemaining = weekly ? n(weekly[2]) : null;

  if (fiveRemaining === null && weekRemaining === null && !credits) {
    throw Object.assign(new Error('No pude parsear Codex usage.'), { code: 'page_changed' });
  }

  return {
    ok: true,
    source: 'playwright',
    updated_at: new Date().toISOString(),
    five_hour: {
      used_percent: fiveRemaining === null ? null : 100 - fiveRemaining,
      remaining_percent: fiveRemaining,
      reset_label: fiveHour ? fiveHour[5].trim() : null,
      reset_epoch: null,
    },
    weekly: {
      used_percent: weekRemaining === null ? null : 100 - weekRemaining,
      remaining_percent: weekRemaining,
      reset_label: weekly ? weekly[5].trim() : null,
      reset_epoch: null,
    },
    credits_remaining: credits ? n(credits[2]) : null,
  };
}

function parseClaude(raw) {
  const text = normalizeText(raw);

  const session = text.match(
    /(Sesión actual|Current session).*?(Se restablece en|Resets in)\s+(.+?)\s+(\d+)\s*%\s*(usado|used)/i,
  );

  const weekly = text.match(
    /(Todos los modelos|All models).*?(Se restablece|Resets)\s+(.+?)\s+(\d+)\s*%\s*(usado|used)/i,
  );

  const routines = text.match(
    /(Ejecuciones de rutinas diarias incluidas|Daily routine executions included).*?(\d+)\s*\/\s*(\d+)/i,
  );

  const sessionUsed = session ? n(session[4]) : null;
  const weeklyUsed = weekly ? n(weekly[4]) : null;

  if (sessionUsed === null && weeklyUsed === null && !routines) {
    throw Object.assign(new Error('No pude parsear Claude usage.'), { code: 'page_changed' });
  }

  return {
    ok: true,
    source: 'playwright',
    updated_at: new Date().toISOString(),
    session: {
      used_percent: sessionUsed,
      remaining_percent: sessionUsed === null ? null : 100 - sessionUsed,
      reset_label: session ? `en ${session[3].trim()}` : null,
      reset_epoch: null,
    },
    weekly: {
      used_percent: weeklyUsed,
      remaining_percent: weeklyUsed === null ? null : 100 - weeklyUsed,
      reset_label: weekly ? weekly[3].trim() : null,
      reset_epoch: null,
    },
    daily_routines: {
      used: routines ? n(routines[2]) : null,
      limit: routines ? n(routines[3]) : null,
    },
  };
}

async function readJson(file, fallback) {
  try {
    return JSON.parse(await fs.readFile(file, 'utf8'));
  } catch {
    return fallback;
  }
}

async function scrapeProvider(name, providerConfig, parser) {
  const profileDir = expandHome(providerConfig.profile_dir);
  try {
    await fs.mkdir(profileDir, { recursive: true, mode: 0o700 });
    await clearStaleSingletonLock(profileDir);

    const text = await withDisplay(async (display) => {
      const context = await chromium.launchPersistentContext(profileDir, {
        // Mismo motivo que en ai-usage-auth.mjs: el Chrome real evita los
        // desafíos "Verify you are human" de Cloudflare que bloquean al
        // Chromium de Playwright.
        channel: 'chrome',
        headless: display.headless ?? providerConfig.headless !== false,
        viewport: { width: 1280, height: 900 },
        args: ['--disable-blink-features=AutomationControlled', ...(display.args ?? [])],
        env: display.env,
      });

      try {
        const page = await context.newPage();
        await page.goto(providerConfig.url, { waitUntil: 'domcontentloaded', timeout: 60_000 });
        await page.waitForTimeout(providerConfig.wait_ms ?? 8000);
        return await page.locator('body').innerText({ timeout: 30_000 });
      } finally {
        await context.close();
      }
    });

    if (/iniciar sesión|log in|sign in|sign up/i.test(text)) {
      throw Object.assign(new Error(`${name} requiere iniciar sesión.`), { code: 'login_required' });
    }

    if (providerConfig.debug_text) {
      await fs.writeFile(path.join(CACHE_DIR, `debug-${name}-text.txt`), text);
    }

    return parser(text);
  } catch (error) {
    return {
      ok: false,
      source: 'playwright',
      updated_at: new Date().toISOString(),
      error_code: error.code ?? 'collector_error',
      error_message: error.message,
    };
  }
}

function buildPrimary(config, status) {
  const providerName = config.primary?.provider ?? 'codex';
  const metricName = config.primary?.metric ?? 'five_hour';
  const provider = status[providerName];
  const metric = provider?.[metricName];
  const used = metric?.used_percent ?? null;
  const remaining = metric?.remaining_percent ?? null;

  return {
    provider: provider?.ok ? providerName : 'none',
    metric: provider?.ok ? metricName : 'none',
    used_percent: used,
    remaining_percent: remaining,
    label: used === null ? '--' : `${Math.round(used)}%`,
  };
}

async function writeAtomic(file, data) {
  const tmp = `${file}.tmp`;
  await fs.writeFile(tmp, JSON.stringify(data, null, 2));
  await fs.rename(tmp, file);
}

async function main() {
  await fs.mkdir(CACHE_DIR, { recursive: true, mode: 0o700 });
  const config = await readJson(CONFIG_FILE, DEFAULT_CONFIG);

  const result = {
    schema_version: 1,
    updated_at: new Date().toISOString(),
    ok: true,
    stale: false,
    primary: null,
    errors: [],
  };

  if (config.providers.codex?.enabled) {
    result.codex = await scrapeProvider('codex', config.providers.codex, parseCodex);
  }

  if (config.providers.claude?.enabled) {
    result.claude = await scrapeProvider('claude', config.providers.claude, parseClaude);
  }

  for (const providerName of ['codex', 'claude']) {
    const provider = result[providerName];
    if (provider && provider.ok === false) {
      result.ok = false;
      result.errors.push({
        provider: providerName,
        code: provider.error_code,
        message: provider.error_message,
      });
    }
  }

  result.primary = buildPrimary(config, result);

  await writeAtomic(OUT_FILE, result);

  if (result.ok) {
    await writeAtomic(LAST_GOOD_FILE, result);
  }

  console.log(JSON.stringify(result, null, 2));
}

main().catch(async (error) => {
  await fs.mkdir(CACHE_DIR, { recursive: true, mode: 0o700 });
  const fallback = {
    schema_version: 1,
    updated_at: new Date().toISOString(),
    ok: false,
    primary: { provider: 'none', metric: 'none', used_percent: null, remaining_percent: null, label: '!' },
    errors: [{ provider: 'collector', code: 'fatal', message: error.message }],
  };
  await writeAtomic(OUT_FILE, fallback);
  console.error(error);
  process.exit(1);
});
