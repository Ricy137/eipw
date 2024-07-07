import {Opts} from "./bindings/Opts";

interface SnippetDef {
    formatted: string;
    [key: string]: any;
}

/**
 * @param {string[]} sources
 * @param {Opts | undefined} options
 * @returns {Promise<any>}
 */
export function lint(sources: string[], options?: Opts): Promise<SnippetDef>;

/**
 * @param {any} snippet
 * @returns {string}
 */
export function format(snippet: SnippetDef[]): string;
