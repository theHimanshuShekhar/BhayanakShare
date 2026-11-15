// Choosing the save folder: how a refused folder is worded.

import type { SaveFolderErrorKind } from "./bindings";
import type { MessageKey } from "./i18n";
import { wordShellError } from "./shellError";

/**
 * The catalogue message (section 6) for each kind of refused folder. `other` has the reason of the
 * system in brackets. A kind added in the shell fails the typecheck until it is worded.
 */
export const SAVE_FOLDER_ERRORS: Record<SaveFolderErrorKind, MessageKey> = {
  not_absolute: "saveFolder.error.notAbsolute",
  not_a_folder: "saveFolder.error.notAFolder",
  cannot_create: "saveFolder.error.cannotCreate",
  not_writable: "saveFolder.error.notWritable",
  not_text: "saveFolder.error.notText",
  not_connected: "saveFolder.error.notConnected",
  other: "saveFolder.failed",
};

/** Why a folder could not be made the save folder, in words for the person using the app. */
export function saveFolderFailure(e: unknown): string {
  return wordShellError(e, SAVE_FOLDER_ERRORS, SAVE_FOLDER_ERRORS.other);
}
