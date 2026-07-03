// Use Prompting Press prompts with LangChain / LangGraph (TypeScript).
//
// Prompting Press renders a composition to an ordered list of role-tagged
// messages. LangChain accepts plain `{ role, content }` objects directly, so
// the bridge is a one-line key rename (`text` -> `content`): map each rendered
// message and hand the list straight to a chat model or a LangGraph node.
//
// Note: do NOT route already-rendered text through
// `ChatPromptTemplate.fromMessages` with the tuple/object shorthand — that path
// treats `content` as a template and breaks on literal `{...}` (e.g. JSON) in
// your rendered text. Prompting Press already did the templating; feed the
// model/graph the messages directly.
import assert from "node:assert/strict";
import { test } from "node:test";
import { z } from "zod";
import { Composition, Prompt } from "prompting-press";
import {
  AIMessage,
  HumanMessage,
  SystemMessage,
  coerceMessageLikeToMessage,
} from "@langchain/core/messages";

const SysVars = z.object({ instructions: z.string().min(1) });
const UserVars = z.object({ payload: z.string().min(1) });

// [{ role, text }] -> [{ role, content }]. Order + role preserved; LangChain
// accepts the role strings (system/user/assistant) as-is.
function toLangchain(messages: { role: string; text: string }[]) {
  return messages.map((m) => ({ role: m.role, content: m.text }));
}

const sysPrompt = new Prompt({
  name: "system-preamble",
  role: "system",
  body: "{{ instructions }}",
  variables: { instructions: { type: "string", trusted: true } },
});
const userPrompt = new Prompt({
  name: "user-turn",
  role: "user",
  body: "{{ payload }}",
  variables: { payload: { type: "string", trusted: false } },
});

test("map a Prompting Press composition to LangChain messages", () => {
  const comp = Composition.fromMessages([
    { prompt: sysPrompt, schema: SysVars, data: { instructions: "You are a helpful assistant." } },
    // Literal braces in the rendered text prove it is NOT re-templated.
    { prompt: userPrompt, schema: UserVars, data: { payload: 'Return this exactly: {"k": 1}' } },
  ]);

  const lcMessages = toLangchain(comp.resolve());

  // Key rename only: order + role preserved, content === text verbatim.
  assert.deepEqual(lcMessages, [
    { role: "system", content: "You are a helpful assistant." },
    { role: "user", content: 'Return this exactly: {"k": 1}' },
  ]);

  // LangChain coerces the objects to the right message classes; the literal
  // braces survive (they would break ChatPromptTemplate's template path).
  const coerced = lcMessages.map(coerceMessageLikeToMessage);
  assert.deepEqual(
    coerced.map((m) => m.constructor.name),
    ["SystemMessage", "HumanMessage"],
  );
  assert.equal(coerced[1].content, 'Return this exactly: {"k": 1}');

  // A real app would then call `await model.invoke(lcMessages)` (ChatOpenAI,
  // ChatBedrock, ...) or seed a LangGraph MessagesState with `lcMessages`.
  // Both accept the role-object list directly — no ChatPromptTemplate needed.
  void [AIMessage, HumanMessage, SystemMessage];
});
