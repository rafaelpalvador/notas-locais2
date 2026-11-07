export {EditorState,EditorSelection,StateField,StateEffect,RangeSetBuilder,Compartment,Transaction,Annotation,Prec,RangeSet} from '@codemirror/state';
export {EditorView,Decoration,WidgetType,ViewPlugin,keymap,drawSelection,highlightActiveLine,placeholder,dropCursor,highlightSpecialChars,rectangularSelection} from '@codemirror/view';
export {history,defaultKeymap,historyKeymap,indentWithTab,undo,redo,insertNewlineAndIndent} from '@codemirror/commands';
export {syntaxTree,syntaxHighlighting,HighlightStyle,defaultHighlightStyle,indentOnInput,bracketMatching,LanguageDescription,LanguageSupport,StreamLanguage,ensureSyntaxTree,foldService} from '@codemirror/language';
export {autocompletion,closeBrackets,closeBracketsKeymap,completionKeymap,startCompletion} from '@codemirror/autocomplete';
export {tags,highlightTree,classHighlighter,tagHighlighter} from '@lezer/highlight';
import {markdown,markdownLanguage,markdownKeymap} from '@codemirror/lang-markdown';
import {LanguageDescription,StreamLanguage} from '@codemirror/language';
import {javascript} from '@codemirror/lang-javascript';
import {python} from '@codemirror/lang-python';
import {html} from '@codemirror/lang-html';
import {css} from '@codemirror/lang-css';
import {json} from '@codemirror/lang-json';
import {sql} from '@codemirror/lang-sql';
import {rust} from '@codemirror/lang-rust';
import {java} from '@codemirror/lang-java';
import {cpp} from '@codemirror/lang-cpp';
import {php} from '@codemirror/lang-php';
import {xml} from '@codemirror/lang-xml';
import {yaml} from '@codemirror/lang-yaml';
import {shell} from '@codemirror/legacy-modes/mode/shell';
import {go} from '@codemirror/legacy-modes/mode/go';
import {csharp,kotlin,dart} from '@codemirror/legacy-modes/mode/clike';
import {swift} from '@codemirror/legacy-modes/mode/swift';
import {ruby} from '@codemirror/legacy-modes/mode/ruby';
import {powerShell} from '@codemirror/legacy-modes/mode/powershell';
import {r} from '@codemirror/legacy-modes/mode/r';
import {lua} from '@codemirror/legacy-modes/mode/lua';
import {toml} from '@codemirror/legacy-modes/mode/toml';
import {dockerFile} from '@codemirror/legacy-modes/mode/dockerfile';
import {GFM,Strikethrough,Table,TaskList} from '@lezer/markdown';
const D=(name,alias,fn)=>LanguageDescription.of({name,alias,support:fn()});
const SL=m=>()=>StreamLanguage.define(m);
export const codeLanguages=[
 D('JavaScript',['js','javascript','jsx','mjs'],()=>javascript({jsx:true})),D('TypeScript',['ts','typescript','tsx'],()=>javascript({typescript:true,jsx:true})),
 D('Python',['py','python'],python),D('HTML',['html','htm','svelte','vue'],html),D('CSS',['css','scss','less'],css),D('JSON',['json','jsonc'],json),
 D('SQL',['sql','postgres','mysql','sqlite'],sql),D('Rust',['rs','rust'],rust),D('Java',['java'],java),D('C++',['c','cpp','c++','h','hpp','cc'],cpp),
 D('PHP',['php'],php),D('XML',['xml','svg'],xml),D('YAML',['yaml','yml'],yaml),
 D('Shell',['sh','bash','zsh','shell','console'],SL(shell)),D('Go',['go','golang'],SL(go)),D('C#',['cs','csharp','c#'],SL(csharp)),D('Kotlin',['kt','kotlin'],SL(kotlin)),
 D('Dart',['dart'],SL(dart)),D('Swift',['swift'],SL(swift)),D('Ruby',['rb','ruby'],SL(ruby)),D('PowerShell',['ps1','powershell','pwsh'],SL(powerShell)),
 D('R',['r'],SL(r)),D('Lua',['lua'],SL(lua)),D('TOML',['toml'],SL(toml)),D('Dockerfile',['dockerfile','docker'],SL(dockerFile))];
export {markdown,markdownLanguage,markdownKeymap,GFM,Strikethrough,Table,TaskList};
