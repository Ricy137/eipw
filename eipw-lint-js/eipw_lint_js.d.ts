import {Opts} from "./bindings/Opts";

interface SnippetDef {
    formatted: string;
    [key: string]: any;
}

/**
 * @param {string[]} sources
 * @param {Opts | undefined} options
 * @returns {Promise<SnippetDef>}
 */
export function lint(sources: string[], options?: Opts): Promise<SnippetDef>;

/**
 * @param {SnippetDef[]} snippet
 * @returns {string}
 */
export function format(snippet: SnippetDef[]): string;
