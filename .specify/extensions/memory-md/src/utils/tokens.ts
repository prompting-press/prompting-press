// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import { get_encoding, Tiktoken } from "@dqbd/tiktoken";

let encoding: Tiktoken | null = null;

function getEncoding(): Tiktoken {
  if (!encoding) {
    encoding = get_encoding("cl100k_base");
  }
  return encoding;
}

export function estimateTokens(text: string): number {
  if (!text.trim()) {
    return 0;
  }
  return getEncoding().encode(text).length;
}

export function freeTokenizer(): void {
  if (encoding) {
    encoding.free();
    encoding = null;
  }
}
