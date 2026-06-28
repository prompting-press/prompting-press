/**
 * GENERATED — DO NOT EDIT.
 *
 * Source of truth: schemas/jsonschema/prompt-definition.schema.json
 * Regenerate with: pnpm -C packages/typescript codegen
 *
 * This file is committed and freshness-gated in CI (constitution C-07).
 * Edit the JSON Schema and regenerate; never hand-edit this file.
 */

/**
 * The single source of truth for a prompt's shape (constitution Principle VII / C-07). Per-language shapes (Pydantic v2 / TypeScript types / Rust serde structs) are code-generated from THIS document; it is never hand-mirrored. The library authors, generates, and round-trips this shape; it does not render, validate, or resolve it (Principle III — that lives in specs 002+). $id is a stable identity, not a live endpoint (research D5).
 */
export interface PromptDefinition {
  /**
   * Logical prompt name; the caller's reference key.
   */
  name: string;
  /**
   * Conversational role; first-class metadata the caller reads. Shared across all variants.
   */
  role: "system" | "user" | "assistant";
  /**
   * The DEFAULT variant's template source. The root body IS the default arm (FR-011); surfaced under reserved name 'default' with is_default=true.
   */
  body: string;
  /**
   * Declared input variables, shared across all variants. Rich enough to generate-then-extend a typed Vars model in a later spec.
   */
  variables?: {
    [k: string]: VariableDecl;
  };
  /**
   * Named alternative arms. Absent => the prompt has only the default (root body) arm. Each arm differs ONLY in body (+ optional opaque meta).
   */
  variants?: {
    [k: string]: Variant;
  };
  /**
   * Optional OPAQUE reference to the caller's output model (e.g. 'NodeOutput'). Stored and echoed; never resolved, loaded, or parsed (Principle III). Shared across variants.
   */
  output_model?: string;
  /**
   * Arbitrary prompt-level metadata; library-OPAQUE (may include uninterpreted model/param hints). Never interpreted by the library.
   */
  metadata?: {
    [k: string]: unknown;
  };
  /**
   * The default (root) arm's selection metadata; library-opaque (weight, group, tags, ...). Symmetric with Variant.meta.
   */
  meta?: {
    [k: string]: unknown;
  };
}
/**
 * A declared input variable: type + origin + optional JSON-Schema validation constraints (carried for generate-then-extend).
 */
export interface VariableDecl {
  /**
   * JSON-Schema type keyword(s) for the variable.
   */
  type:
    | ("string" | "integer" | "number" | "boolean" | "array" | "object")
    | ("string" | "integer" | "number" | "boolean" | "array" | "object" | "null")[];
  /**
   * Per-field origin (input-trust) tag (FR-010a; renamed from `provenance` in spec 008). DECLARATIVE METADATA ONLY — there is NO runtime enforcement of this tag in the current library version; it is not a security guard by itself. Untrusted-input guarding (the opt-in, additive guard expansion + lint) is introduced in a later spec per roadmap decision C-09 (deriving from constitution Principle IV). Do not assume the library protects `untrusted`/`external` fields until that version. NOTE: this is the per-VARIABLE trust tag, distinct from the render-result provenance (template_hash/render_hash) which is unchanged.
   */
  origin: "trusted" | "untrusted" | "external";
  /**
   * When true, a validator covering this variable MUST be supplied when the Prompt is constructed (spec 008). Orthogonal to `origin` — it MAY mark any variable, not only untrusted/external ones. Declarative metadata; enforcement is per-language (constitution Principle VI v1.2.0): TypeScript (Zod) and Python (Pydantic) introspect the supplied validator and throw/raise at construction if this variable is uncovered, while Rust guarantees coverage structurally at compile time. The kernel never reads this field (validation-blind).
   */
  validation_required?: boolean;
  format?: string;
  pattern?: string;
  enum?: unknown[];
  minimum?: number;
  maximum?: number;
  minLength?: number;
  maxLength?: number;
  description?: string;
}
/**
 * A named alternative arm. May carry ONLY body and meta; redefining role/variables/output_model is rejected (FR-011a).
 */
export interface Variant {
  /**
   * The variant's template source — the only field that differs per variant.
   */
  body: string;
  /**
   * Library-OPAQUE selection metadata (weight, group, tags, ...). Stored + exposed; never interpreted by the library (caller selects). No schema-enforced selection semantics (FR-011c).
   */
  meta?: {
    [k: string]: unknown;
  };
}
