// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

// Use Prompting Press prompts with the Strands Agents SDK (TypeScript).
//
// Strands has no `system` role in its message list: `role` is exactly
// `"user" | "assistant"`, and the system prompt is a SEPARATE Agent argument.
// So the bridge PARTITIONS a Prompting Press composition:
//
//   * system-role texts -> a single `systemPrompt` string (joined with "\n\n")
//   * everything else    -> `messages` as { role, content: [{ text }] }
//
// Strands cannot preserve the POSITION of a system message inside the
// conversation — every system-role text is hoisted into the one system prompt.
// That is a Strands limitation, not something Prompting Press can carry. Only
// plain text is mapped; Strands' provider-specific content blocks
// (guardContent, toolResult, ...) are out of scope here.
import assert from "node:assert/strict";
import { test } from "node:test";
import { z } from "zod";
import { Composition, Prompt } from "prompting-press";
import { Agent } from "@strands-agents/sdk";

const TextVars = z.object({ text: z.string().min(1) });

type Msg = { role: string; text: string };

// Returns the separate system prompt (undefined if none) + the Strands messages.
function toStrands(messages: Msg[]): {
  system: string | undefined;
  convo: { role: "user" | "assistant"; content: { text: string }[] }[];
} {
  const systemTexts = messages.filter((m) => m.role === "system").map((m) => m.text);
  const system = systemTexts.length ? systemTexts.join("\n\n") : undefined;
  const convo = messages
    .filter((m) => m.role !== "system")
    .map((m) => ({ role: m.role as "user" | "assistant", content: [{ text: m.text }] }));
  return { system, convo };
}

function prompt(name: string, role: "system" | "user" | "assistant", trusted = true) {
  return new Prompt({
    name,
    role,
    body: "{{ text }}",
    variables: { text: { type: "string", trusted } },
  });
}

test("partition a Prompting Press composition for Strands", () => {
  // TWO system messages (exercises the "\n\n"-join + ordering), then a
  // user/assistant/user exchange.
  const comp = Composition.fromMessages([
    { prompt: prompt("sys-1", "system"), schema: TextVars, data: { text: "You are a support agent." } },
    { prompt: prompt("sys-2", "system"), schema: TextVars, data: { text: "Answer only in English." } },
    { prompt: prompt("u-1", "user", false), schema: TextVars, data: { text: "What's your return policy?" } },
    { prompt: prompt("a-1", "assistant"), schema: TextVars, data: { text: "30 days, unopened." } },
    { prompt: prompt("u-2", "user", false), schema: TextVars, data: { text: "And opened items?" } },
  ]);

  const { system, convo } = toStrands(comp.resolve());

  // Both system texts hoisted, joined in order with a blank line between.
  assert.equal(system, "You are a support agent.\n\nAnswer only in English.");

  // convo drops the system messages; only user/assistant remain, in order,
  // each wrapped as a single { text } content block.
  assert.deepEqual(convo, [
    { role: "user", content: [{ text: "What's your return policy?" }] },
    { role: "assistant", content: [{ text: "30 days, unopened." }] },
    { role: "user", content: [{ text: "And opened items?" }] },
  ]);

  // Construct the agent from the two seams (no invocation — offline).
  const agent = new Agent({ systemPrompt: system, messages: convo });
  assert.equal(agent.systemPrompt, system);
  assert.equal(agent.messages.length, 3);
  assert.deepEqual(
    agent.messages.map((m) => m.role),
    ["user", "assistant", "user"],
  );
});
