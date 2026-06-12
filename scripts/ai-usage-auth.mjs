#!/usr/bin/env node
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline/promises';

const HOME = os.homedir();
const provider = process.argv[2];

const providers = {
  codex: {
    profileDir: path.join(HOME, '.config', 'ai-usage-status', 'profiles', 'chatgpt'),
    url: 'https://chatgpt.com/codex/cloud/settings/analytics#usage',
    validate: /Límite de uso|usage limit|Créditos restantes|Credits remaining/i,
  },
  claude: {
    profileDir: path.join(HOME, '.config', 'ai-usage-status', 'profiles', 'claude'),
    url: 'https://claude.ai/settings/usage',
    validate: /Sesión actual|Current session|Todos los modelos|All models|Usage/i,
  },
};

if (!providers[provider]) {
  console.error('Uso: ai-usage-auth codex|claude');
  process.exit(1);
}

const selected = providers[provider];
await fs.mkdir(selected.profileDir, { recursive: true, mode: 0o700 });

const context = await chromium.launchPersistentContext(selected.profileDir, {
  headless: false,
  viewport: { width: 1280, height: 900 },
});

const page = await context.newPage();
await page.goto(selected.url, { waitUntil: 'domcontentloaded', timeout: 60_000 });

console.log(`\nInicia sesión en ${provider}.`);
console.log('Cuando veas el panel de uso cargado, vuelve a esta terminal y presiona Enter.\n');

const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
await rl.question('Presiona Enter para validar sesión...');
rl.close();

await page.waitForTimeout(2000);
const text = await page.locator('body').innerText({ timeout: 30_000 }).catch(() => '');

if (!selected.validate.test(text)) {
  console.error(`No pude confirmar que estés en la pantalla de uso de ${provider}.`);
  console.error('La sesión pudo guardarse igual, pero debes revisar URL/textos/idioma.');
  await context.close();
  process.exit(2);
}

await context.close();
console.log(`Login guardado correctamente para ${provider}.`);
