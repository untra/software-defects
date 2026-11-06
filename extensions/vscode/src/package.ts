declare const STATE_LENS_LANGUAGE: string | undefined;
declare const STATE_LENS_EXTENSIONS: string[] | undefined;
export const LANGUAGE = typeof STATE_LENS_LANGUAGE === 'undefined' ? 'typescript' : STATE_LENS_LANGUAGE;
export const EXTENSIONS = typeof STATE_LENS_EXTENSIONS === 'undefined' ? ['ts'] : STATE_LENS_EXTENSIONS;
export const CONFIG_SECTION = `stateLens.${LANGUAGE}`;
export const commandId = (name: string): string => `${CONFIG_SECTION}.${name}`;
