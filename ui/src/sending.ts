// Offering files: how files the Device could not offer are worded.

import type { SendErrorKind } from "./bindings";
import type { MessageKey } from "./i18n";
import { wordShellError } from "./shellError";

/**
 * The catalogue message for each reason files could not be offered: C8 for a selection over the
 * Offer's limits, else the reason in brackets. A kind added in the shell fails the typecheck
 * until it is worded.
 */
export const SEND_ERRORS: Record<SendErrorKind, MessageKey> = {
  too_big: "send.tooBig",
  other: "send.failed",
};

/**
 * Why an Offer could not be sent, in words for the person using the app. `other` words a reason
 * that is not one of the catalogue's own, with the reason in brackets: starting a Transfer again
 * says so.
 */
export function sendFailure(e: unknown, other: MessageKey = SEND_ERRORS.other): string {
  return wordShellError(e, { ...SEND_ERRORS, other }, other);
}
