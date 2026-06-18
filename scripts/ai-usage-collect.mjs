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

// Días de la semana (es/en, con y sin acento) → índice 0=domingo..6=sábado.
const WEEKDAYS = {
  dom: 0, lun: 1, mar: 2, mié: 3, mie: 3, jue: 4, vie: 5, sáb: 6, sab: 6,
  sun: 0, mon: 1, tue: 2, wed: 3, thu: 4, fri: 5, sat: 6,
};

/** Extrae `{ h, m }` (24h) de un reloj tipo `7:12 AM` o `11:59 p.m.`. */
function parseClock(text) {
  const match = text.match(/(\d{1,2}):(\d{2})\s*([ap])\s*\.?\s*m/i) || text.match(/(\d{1,2}):(\d{2})/);
  if (!match) return null;
  let h = Number(match[1]);
  const m = Number(match[2]);
  if (match[3]) {
    const pm = /p/i.test(match[3]);
    if (pm && h < 12) h += 12;
    if (!pm && h === 12) h = 0;
  }
  return { h, m };
}

/**
 * Convierte una etiqueta "reinicia" en un timestamp Unix (segundos), para que
 * el applet muestre los minutos restantes en vivo. Maneja cuatro formatos:
 *   - Duración:        "en 4 h 34 min"        → ahora + duración
 *   - Fecha completa:  "Jun 25, 2026 2:12 AM" → Date.parse (locale + tz local)
 *   - Día + hora:      "mié., 11:59 p.m."     → próxima ocurrencia de ese día
 *   - Solo hora:       "7:12 AM"              → hoy, o mañana si ya pasó
 * Devuelve null si no puede inferir un instante (nunca inventa una fecha).
 */
function resetEpochSeconds(label, now = new Date()) {
  if (!label) return null;
  const text = String(label).trim();
  const nowSec = Math.round(now.getTime() / 1000);

  const dur = text.match(
    /^(?:en\s+)?(?:(\d+)\s*(?:d|días?|dias?|days?)\s*)?(?:(\d+)\s*(?:h|hr|horas?|hours?)\s*)?(?:(\d+)\s*(?:min|mins?|minutos?)\s*)?$/i,
  );
  if (dur && (dur[1] || dur[2] || dur[3])) {
    const mins = Number(dur[1] || 0) * 1440 + Number(dur[2] || 0) * 60 + Number(dur[3] || 0);
    return nowSec + mins * 60;
  }

  if (/\d{4}/.test(text)) {
    const parsed = Date.parse(text);
    if (!Number.isNaN(parsed)) return Math.round(parsed / 1000);
  }

  const clock = parseClock(text);
  if (!clock) return null;

  // Anclado al inicio: la etiqueta de día empieza con el día ("mié., 11:59
  // p.m."). No se usa \b porque las vocales acentuadas (mié, sáb) no cuentan
  // como "word char" en JS y romperían el límite de palabra.
  const weekday = text.toLowerCase().match(/^(dom|lun|mar|mié|mie|jue|vie|sáb|sab|sun|mon|tue|wed|thu|fri|sat)/);
  const target = new Date(now);
  target.setHours(clock.h, clock.m, 0, 0);

  if (weekday) {
    let add = (WEEKDAYS[weekday[1]] - target.getDay() + 7) % 7;
    if (add === 0 && target.getTime() <= now.getTime()) add = 7;
    target.setDate(target.getDate() + add);
  } else if (target.getTime() <= now.getTime()) {
    target.setDate(target.getDate() + 1);
  }

  return Math.round(target.getTime() / 1000);
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
      reset_epoch: fiveHour ? resetEpochSeconds(fiveHour[5].trim()) : null,
    },
    weekly: {
      used_percent: weekRemaining === null ? null : 100 - weekRemaining,
      remaining_percent: weekRemaining,
      reset_label: weekly ? weekly[5].trim() : null,
      reset_epoch: weekly ? resetEpochSeconds(weekly[5].trim()) : null,
    },
    credits_remaining: credits ? n(credits[2]) : null,
  };
}

/**
 * Sección "Créditos de uso" del panel de Claude. Solo existe si el usuario
 * la activó, así que todos los campos son opcionales y un fallo de parseo
 * devuelve `null` sin afectar el resto del status. Las regex se anclan a sus
 * propias etiquetas (no al orden) y el guard `(?!USD)` evita cruzar otro
 * monto, para tolerar variaciones de layout entre cuentas/idiomas.
 */
function parseClaudeCredits(text) {
  if (!/(Créditos de uso|Usage credits)/i.test(text)) {
    return null;
  }

  const spent = text.match(/USD\s*([\d.,]+)\s*(?:gastado|spent)\b/i);
  const spentPercent = text.match(
    /USD\s*[\d.,]+\s*(?:gastado|spent)\b[\s\S]*?(\d+)\s*%\s*(?:usado|used)/i,
  );
  // El label del reset termina antes del siguiente porcentaje o monto, para no
  // tragarse el "16% usado" que el DOM intercala (p. ej. "Jul 1 16% usado USD").
  const reset = text.match(
    /(?:gastado|spent)\b[\s\S]*?(?:Se restablece(?:\s+el)?|Resets(?:\s+on)?)\s+(.+?)\s+(?:\d+\s*%|USD\b)/i,
  );
  const limit = text.match(
    /USD\s*([\d.,]+)(?:(?!USD)[\s\S]){0,40}?(?:Límite de gasto mensual|Monthly spending limit)/i,
  );
  // El saldo puede aparecer como "USD 9.51 ... Saldo actual" o al revés, según
  // el orden del DOM; se prueban ambos sentidos.
  const balance =
    text.match(/USD\s*([\d.,]+)(?:(?!USD)[\s\S]){0,40}?(?:Saldo actual|Current balance)/i) ||
    text.match(/(?:Saldo actual|Current balance)(?:(?!USD)[\s\S]){0,60}?USD\s*([\d.,]+)/i);

  return {
    enabled: true,
    spent_usd: spent ? n(spent[1]) : null,
    spent_percent: spentPercent ? n(spentPercent[1]) : null,
    limit_usd: limit ? n(limit[1]) : null,
    balance_usd: balance ? n(balance[1]) : null,
    reset_label: reset ? reset[1].trim() : null,
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
      reset_epoch: session ? resetEpochSeconds(`en ${session[3].trim()}`) : null,
    },
    weekly: {
      used_percent: weeklyUsed,
      remaining_percent: weeklyUsed === null ? null : 100 - weeklyUsed,
      reset_label: weekly ? weekly[3].trim() : null,
      reset_epoch: weekly ? resetEpochSeconds(weekly[3].trim()) : null,
    },
    daily_routines: {
      used: routines ? n(routines[2]) : null,
      limit: routines ? n(routines[3]) : null,
    },
    credits: parseClaudeCredits(text),
  };
}

async function readJson(file, fallback) {
  try {
    return JSON.parse(await fs.readFile(file, 'utf8'));
  } catch {
    return fallback;
  }
}

/**
 * Espera a que el texto de uso esté presente en la página y lo devuelve
 * apenas aparece, en vez de esperar siempre `maxMs` fijos. Sondea cada
 * `POLL_MS` y se rinde a los `maxMs` (devolviendo lo último que haya, así
 * la detección de login_required sigue funcionando). Esto reduce el tiempo
 * típico de scraping de ~8 s a ~1-3 s sin tocar el techo de seguridad, lo
 * que abarata cada corrida del recolector.
 */
const POLL_MS = 250;
async function waitForUsageText(page, readyPattern, maxMs) {
  const deadline = Date.now() + maxMs;
  let text = '';
  for (;;) {
    text = await page.locator('body').innerText({ timeout: 30_000 });
    if (readyPattern.test(text) || Date.now() >= deadline) {
      return text;
    }
    await page.waitForTimeout(POLL_MS);
  }
}

async function scrapeProvider(name, providerConfig, parser, readyPattern) {
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
        return await waitForUsageText(page, readyPattern, providerConfig.wait_ms ?? 8000);
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

  // Marcadores que solo aparecen cuando el panel de uso ya cargó, para que
  // `waitForUsageText` corte apenas haya datos en vez de esperar el tope.
  const CODEX_READY = /(Límite de uso|usage limit|Créditos restantes|Credits remaining)/i;
  const CLAUDE_READY = /(Sesión actual|Current session|Todos los modelos|All models)/i;

  if (config.providers.codex?.enabled) {
    result.codex = await scrapeProvider('codex', config.providers.codex, parseCodex, CODEX_READY);
  }

  if (config.providers.claude?.enabled) {
    result.claude = await scrapeProvider('claude', config.providers.claude, parseClaude, CLAUDE_READY);
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
