// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

/**
 * Greeter CLI — a realistic Prompting Press consumer sample app (spec 014, WU-C, T015).
 *
 * Walks the FULL public feature surface end-to-end (FR-014): construct → validate →
 * render default + a named variant → compose a 2-message prompt → check() → the
 * advisory guard → provenance hashes → an error path. The "hand to an LLM" step is a
 * printed stub — the library never calls a provider (FR-018).
 *
 * Run it: pnpm start  (after pnpm build)
 */

import {
  Composition,
  Prompt,
  PromptRenderError,
} from "prompting-press";
import { z } from "zod";

// ── Typed vars schemas (validated by Zod before any templating) ──────────────

const GreetVars = z.object({
  name: z.string().min(1),
  count: z.number().int().min(0),
});

const AskVars = z.object({
  topic: z.string().min(1),
});

const SysVars = z.object({
  instruction: z.string().min(1),
});

// ── Prompt documents (a real consumer would read these from files) ───────────

/** A greeting prompt with a `formal` variant, both sharing the same variables. */
const GREET_YAML = `
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages."
variables:
  name:
    type: string
    trusted: true
  count:
    type: integer
    trusted: true
variants:
  formal:
    body: "Good day, {{ name }}. You have {{ count }} messages awaiting your attention."
`;

/** A prompt with an UNTRUSTED variable, used to demonstrate the guard + check(). */
const ASK_YAML = `
name: ask
role: user
body: "Tell me about {{ topic }}."
variables:
  topic:
    type: string
    trusted: false
`;

/**
 * Run the full feature walk, throwing only on an *unexpected* failure
 * (the demonstrated error path is caught and reported inline, not propagated).
 */
export function run(): void {
  console.log("=== Prompting Press — TypeScript consumer sample ===\n");

  // 1. CONSTRUCT (validates the template↔variables agreement immediately).
  //    Exercise all four construction forms per FR-014.
  const greet = Prompt.fromYaml(GREET_YAML);
  console.log(`[construct] loaded prompt "${greet.name}"`);
  console.log(`[construct] variants: ${JSON.stringify(Object.keys(greet.variants ?? {}))}`);

  // Also exercise fromJson and fromToml (agreement: they all parse the same shape).
  const greetJson = Prompt.fromJson(
    JSON.stringify({
      name: "g",
      role: "user",
      body: "Hi {{ n }}",
      variables: { n: { type: "string", trusted: true } },
    }),
  );
  const greetToml =
    'name = "g"\nrole = "user"\nbody = "Hi {{ n }}"\n[variables.n]\ntype = "string"\ntrusted = true\n';
  const fromToml = Prompt.fromToml(greetToml);
  console.log(
    `[construct:formats] json.body="${greetJson.body}" toml.body="${fromToml.body}"`,
  );

  // Also exercise new Prompt(shape) object form.
  const fromObj = new Prompt({
    name: "greet_obj",
    role: "user",
    body: "Hi {{ name }}, you have {{ count }} messages.",
    variables: {
      name: { type: "string", trusted: true },
      count: { type: "integer", trusted: true },
    },
  });
  console.log(`[construct:new] name="${fromObj.name}"`);

  // 2. VALIDATE + RENDER the default arm.
  const vars = { name: "Ada", count: 3 };
  const defaultResult = greet.render(GreetVars, vars);
  console.log(`\n[render:default] ${defaultResult.text}`);

  // 3. RENDER a named variant — a different body from the same vars.
  const formal = greet.render(GreetVars, vars, { variant: "formal" });
  console.log(`[render:formal]  ${formal.text}`);

  // 4. PROVENANCE — content-addressed hashes on the result.
  console.log(
    `\n[provenance] variant=${defaultResult.variant} template_hash=${defaultResult.templateHash.slice(0, 8)}… render_hash=${defaultResult.renderHash.slice(0, 8)}…`,
  );

  // 5. COMPOSE a 2-message prompt (system preamble + the greeting).
  const sys = Prompt.fromYaml(
    'name: sys\nrole: system\nbody: "{{ instruction }}"\nvariables:\n  instruction:\n    type: string\n    trusted: true\n',
  );
  const comp = new Composition();
  comp.append({ prompt: sys, schema: SysVars, data: { instruction: "Be concise." } });
  comp.append({ prompt: greet, schema: GreetVars, data: vars });
  const messages = comp.resolve();
  console.log(`\n[compose] ${messages.length} messages:`);
  for (const m of messages) {
    console.log(`  ${m.role}: ${m.text}`);
  }

  // 6. CHECK — the advisory lint. `ask` declares an untrusted var with no guard
  //    metadata, so check() surfaces one finding.
  const ask = Prompt.fromYaml(ASK_YAML);
  const report = ask.check();
  console.log(
    `\n[check] ask.check() passed=${report.passed()} findings=${report.findings.length}`,
  );
  for (const f of report.findings) {
    console.log(`  ${f.kind}: ${f.detail}`);
  }

  // 7. GUARD — enable it: the untrusted value is delimited in the body and an
  //    advisory is returned. The library never sends this anywhere.
  const guarded = ask.render(AskVars, { topic: "rivers" }, { guard: { enabled: true } });
  console.log(`\n[guard] text  = ${guarded.text}`);
  console.log(`[guard] guard = ${guarded.guard ?? "<none>"}`);

  // Guard absent/null → no wrap, no advisory.
  const plain = ask.render(AskVars, { topic: "rivers" });
  console.log(`[guard:off] text  = ${plain.text}`);
  console.log(`[guard:off] guard = ${plain.guard ?? "<none>"}`);

  // 8. ERROR PATH — an unknown variant fails loudly with a structured PromptRenderError.
  try {
    greet.render(GreetVars, vars, { variant: "nonexistent" });
    throw new Error("expected the unknown-variant render to fail");
  } catch (err) {
    if (err instanceof PromptRenderError) {
      const row = err.errors[0];
      console.log(
        `\n[error] unknown variant rejected: code=${row?.code ?? "?"} field=${row?.field ?? "?"}`,
      );
    } else {
      throw err;
    }
  }

  // 9. HAND-OFF STUB — a real app would send `messages` to a provider here.
  //    The library does no I/O and calls no model; this is a printed placeholder.
  console.log(
    `\n[handoff] (stub) would POST ${messages.length} messages to the configured LLM provider.`,
  );

  console.log("\n=== done ===");
}

run();
