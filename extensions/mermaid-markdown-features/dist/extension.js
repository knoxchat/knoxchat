"use strict";var R=Object.create;var h=Object.defineProperty;var q=Object.getOwnPropertyDescriptor;var B=Object.getOwnPropertyNames;var F=Object.getPrototypeOf,V=Object.prototype.hasOwnProperty;var N=(e,i)=>{for(var t in i)h(e,t,{get:i[t],enumerable:!0})},S=(e,i,t,r)=>{if(i&&typeof i=="object"||typeof i=="function")for(let o of B(i))!V.call(e,o)&&o!==t&&h(e,o,{get:()=>i[o],enumerable:!(r=q(i,o))||r.enumerable});return e};var _=(e,i,t)=>(t=e!=null?R(F(e)):{},S(i||!e||!e.__esModule?h(t,"default",{value:e,enumerable:!0}):t,e)),J=e=>S(h({},"__esModule",{value:!0}),e);var re={};N(re,{activate:()=>ie});module.exports=J(re);var m=_(require("vscode"));var d=_(require("vscode"));function T(){if(typeof crypto.randomUUID=="function")return crypto.randomUUID.bind(crypto)();let e=new Uint8Array(16),i=[];for(let o=0;o<256;o++)i.push(o.toString(16).padStart(2,"0"));crypto.getRandomValues(e),e[6]=e[6]&15|64,e[8]=e[8]&63|128;let t=0,r="";return r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r+="-",r+=i[e[t++]],r+=i[e[t++]],r+="-",r+=i[e[t++]],r+=i[e[t++]],r+="-",r+=i[e[t++]],r+=i[e[t++]],r+="-",r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r+=i[e[t++]],r}function $(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function Y(e){for(;e.length;)e.pop()?.dispose()}var b=class{_isDisposed=!1;_disposables=[];dispose(){this._isDisposed||(this._isDisposed=!0,Y(this._disposables))}_register(i){return this._isDisposed?i.dispose():this._disposables.push(i),i}get isDisposed(){return this._isDisposed}};var U=_(require("vscode")),u="markdown-mermaid";var G="vscode",K=["vscode","base","forest","dark","default","neutral"];function D(e){return typeof e=="string"&&K.includes(e)?e:G}function Q(){let e=U.workspace.getConfiguration(u);return{darkModeTheme:D(e.get("darkModeTheme")),lightModeTheme:D(e.get("lightModeTheme")),maxTextSize:e.get("maxTextSize"),clickDrag:e.get("mouseNavigation.enabled","alt"),showControls:e.get("controls.show","onHoverOrFocus"),resizable:e.get("resizable",!0),maxHeight:e.get("maxHeight","")}}function z(e){let i=e.renderer.render;return e.renderer.render=function(...t){return`${x()}
				${i.apply(e.renderer,t)}`},e}function x(){let e=X(JSON.stringify(Q()));return`<span id="${u}" aria-hidden="true" data-config="${e}"></span>`}function X(e){return e.replace(/&/g,"&amp;").replace(/"/g,"&quot;").replace(/</g,"&lt;").replace(/>/g,"&gt;")}var A="vscode.mermaid-markdown-features.preview",f=class extends b{constructor(t,r){super();this._extensionUri=t;this._webviewManager=r;this._register(d.window.registerWebviewPanelSerializer(A,this))}_extensionUri;_webviewManager;_previews=new Map;openPreview(t,r){let o=E(t),n=this._previews.get(o);if(n){n.reveal();return}let a=k.create(o,t,r,this._extensionUri,this._webviewManager,d.ViewColumn.Active);this._registerPreview(a)}async deserializeWebviewPanel(t,r){if(!r?.mermaidSource){t.webview.html=this._getErrorHtml();return}let o=E(r.mermaidSource),n=k.revive(t,o,r.mermaidSource,this._extensionUri,this._webviewManager);this._registerPreview(n)}_registerPreview(t){this._previews.set(t.diagramId,t),t.onDispose(()=>{this._previews.delete(t.diagramId)})}_getErrorHtml(){return`<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<title>Mermaid Preview</title>
				<meta http-equiv="Content-Security-Policy" content="default-src 'none';">
				<style>
					body {
						display: flex;
						justify-content: center;
						align-items: center;
						height: 100vh;
						margin: 0;
					}
				</style>
			</head>
			<body>
				<p>An unexpected error occurred while restoring the Mermaid preview.</p>
			</body>
			</html>`}dispose(){super.dispose();for(let t of this._previews.values())t.dispose();this._previews.clear()}},k=class e extends b{constructor(t,r,o,n,a){super();this._webviewPanel=t;this.diagramId=r;this._mermaidSource=o;this._extensionUri=n;this._webviewManager=a;this._webviewPanel.iconPath=new d.ThemeIcon("graph"),this._webviewPanel.webview.options={enableScripts:!0,localResourceRoots:[d.Uri.joinPath(this._extensionUri,"diagram-webview-out")]},this._webviewPanel.webview.html=this._getHtml(),this._register(this._webviewManager.registerWebview(this.diagramId,this._webviewPanel.webview,this._mermaidSource,void 0)),this._register(this._webviewPanel.onDidChangeViewState(v=>{v.webviewPanel.active&&this._webviewManager.setActiveWebview(this.diagramId)})),this._register(this._webviewPanel.onDidDispose(()=>{this._onDisposeEmitter.fire(),this.dispose()}))}_webviewPanel;diagramId;_mermaidSource;_extensionUri;_webviewManager;_onDisposeEmitter=this._register(new d.EventEmitter);onDispose=this._onDisposeEmitter.event;static create(t,r,o,n,a,v){let p=d.window.createWebviewPanel(A,o??d.l10n.t("Mermaid Diagram"),v,{retainContextWhenHidden:!1});return new e(p,t,r,n,a)}static revive(t,r,o,n,a){return new e(t,r,o,n,a)}reveal(){this._webviewPanel.reveal()}dispose(){this._onDisposeEmitter.fire(),super.dispose(),this._webviewPanel.dispose()}_getHtml(){let t=T(),r=d.Uri.joinPath(this._extensionUri,"diagram-webview-out"),o=this._webviewPanel.webview.asWebviewUri(d.Uri.joinPath(r,"index-editor.js")),n=this._webviewPanel.webview.asWebviewUri(d.Uri.joinPath(r,"codicon.css")),a=d.l10n.t("Toggle Pan Mode"),v=d.l10n.t("Zoom Out"),p=d.l10n.t("Zoom In"),s=d.l10n.t("Reset Pan and Zoom");return`<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">
				<meta name="viewport" content="width=device-width, initial-scale=1.0">
				<title>Mermaid Diagram</title>
				<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; script-src 'nonce-${t}'; style-src ${this._webviewPanel.webview.cspSource} 'unsafe-inline'; font-src data:;" />
				<link rel="stylesheet" type="text/css" href="${n}">
				<style>
					html, body {
						margin: 0;
						padding: 0;
						height: 100%;
						width: 100%;
						overflow: hidden;
					}
					.mermaid {
						visibility: hidden;
					}
					.mermaid.rendered {
						visibility: visible;
					}
					.mermaid-wrapper {
						height: 100%;
						width: 100%;
					}
					.zoom-controls {
						position: absolute;
						top: 8px;
						right: 8px;
						display: flex;
						gap: 2px;
						z-index: 100;
						background: var(--vscode-editorWidget-background);
						border: 1px solid var(--vscode-editorWidget-border);
						border-radius: 6px;
						padding: 3px;
					}
					.zoom-controls button {
						display: flex;
						align-items: center;
						justify-content: center;
						width: 26px;
						height: 26px;
						background: transparent;
						color: var(--vscode-icon-foreground);
						border: none;
						border-radius: 4px;
						cursor: pointer;
					}
					.zoom-controls button:hover {
						background: var(--vscode-toolbar-hoverBackground);
					}
					.zoom-controls button.active {
						background: var(--vscode-toolbar-activeBackground);
						color: var(--vscode-focusBorder);
					}
				</style>
			</head>
			<body data-vscode-context='${JSON.stringify({preventDefaultContextMenuItems:!0,mermaidWebviewId:this.diagramId})}' data-vscode-mermaid-webview-id="${this.diagramId}">
				${x()}
				<div class="zoom-controls">
					<button class="pan-mode-btn" title="${a}" aria-label="${a}" aria-pressed="false"><i class="codicon codicon-move" aria-hidden="true"></i></button>
					<button class="zoom-out-btn" title="${v}" aria-label="${v}"><i class="codicon codicon-zoom-out" aria-hidden="true"></i></button>
					<button class="zoom-in-btn" title="${p}" aria-label="${p}"><i class="codicon codicon-zoom-in" aria-hidden="true"></i></button>
					<button class="zoom-reset-btn" title="${s}" aria-label="${s}"><i class="codicon codicon-screen-normal" aria-hidden="true"></i></button>
				</div>
				<pre class="mermaid">
					${$(this._mermaidSource)}
				</pre>
				<script type="module" nonce="${t}" src="${o}"></script>
			</body>
			</html>`}};function E(e){let i=0;for(let t=0;t<e.length;t++){let r=e.charCodeAt(t);i=(i<<5)-i+r,i=i&i}return Math.abs(i).toString(16)}var y="mermaid",I="mermaidContainer";function L(e,i){e.use(r=>{function o(n,a,v,p){let s,P=!1,c=n.bMarks[a]+n.tShift[a],g=n.eMarks[a];if(n.src.charCodeAt(c)!==58)return!1;for(s=c+1;s<=g&&":"[(s-c)%1]===n.src[s];s++);let W=Math.floor((s-c)/1);if(W<3)return!1;s-=(s-c)%1;let O=n.src.slice(c,s),C=n.src.slice(s,g);if(C.trim().split(" ")[0].toLowerCase()!==y)return!1;if(p)return!0;let l=a;for(;l++,!(l>=v||(c=n.bMarks[l]+n.tShift[l],g=n.eMarks[l],c<g&&n.sCount[l]<n.blkIndent));)if(n.src.charCodeAt(c)===58&&!(n.sCount[l]-n.blkIndent>=4)){for(s=c+1;s<=g&&":"[(s-c)%1]===n.src[s];s++);if(!(Math.floor((s-c)/1)<W)&&(s-=(s-c)%1,s=n.skipSpaces(s),!(s<g))){P=!0;break}}let j=n.parentType,Z=n.lineMax;n.parentType="container",n.lineMax=l;let w=n.push(I,"div",1);return w.markup=O,w.block=!0,w.info=C,w.map=[a,l],w.content=n.getLines(a+1,l,n.blkIndent,!0),n.parentType=j,n.lineMax=Z,n.line=l+(P?1:0),!0}r.block.ruler.before("fence",I,o,{alt:["paragraph","reference","blockquote","list"]}),r.renderer.rules[I]=(n,a)=>{let p=n[a].content;return`<div class="${y}">${H(p)}</div>`}});let t=e.options.highlight;return e.options.highlight=(r,o,n)=>{let a=new RegExp("\\b("+i.languageIds().map(ee).join("|")+")\\b","i");return o&&a.test(o)?`<pre class="${y}" style="all: unset;">${H(r)}</pre>`:t?.(r,o,n)??r},e}function H(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/\n+$/,"").trimStart()}function ee(e){return e.replace(/[.*+?^${}()|[\]\\]/g,"\\$&")}var M=class{_activeWebviewId;_webviews=new Map;get activeWebview(){return this._activeWebviewId?this._webviews.get(this._activeWebviewId):void 0}registerWebview(i,t,r,o){if(this._webviews.has(i))throw new Error(`Webview with id ${i} is already registered.`);let n={id:i,webview:t,mermaidSource:r,title:o};return this._webviews.set(i,n),{dispose:()=>this.unregisterWebview(i)}}unregisterWebview(i){this._webviews.delete(i),this._activeWebviewId===i&&(this._activeWebviewId=void 0)}setActiveWebview(i){this._webviews.has(i)&&(this._activeWebviewId=i)}getWebview(i){return this._webviews.get(i)}resetPanZoom(i){(i?this._webviews.get(i):this.activeWebview)?.webview.postMessage({type:"resetPanZoom"})}};function ie(e){let i=new M,t=new f(e.extensionUri,i);return e.subscriptions.push(t),e.subscriptions.push(m.commands.registerCommand("_mermaid-markdown.resetPanZoom",r=>{i.resetPanZoom(r?.mermaidWebviewId)})),e.subscriptions.push(m.commands.registerCommand("_mermaid-markdown.copySource",r=>{if(typeof r?.mermaidSource=="string"){m.env.clipboard.writeText(r.mermaidSource);return}let o=r?.mermaidWebviewId?i.getWebview(r.mermaidWebviewId):i.activeWebview;o&&m.env.clipboard.writeText(o.mermaidSource)})),e.subscriptions.push(m.workspace.onDidChangeConfiguration(r=>{r.affectsConfiguration(`${u}.languages`)&&m.commands.executeCommand("markdown.api.reloadPlugins"),(r.affectsConfiguration(u)||r.affectsConfiguration("workbench.colorTheme"))&&m.commands.executeCommand("markdown.preview.refresh")})),{extendMarkdownIt(r){return L(r,{languageIds:()=>m.workspace.getConfiguration(u).get("languages",["mermaid"])}),r.use(z),r}}}0&&(module.exports={activate});
//# sourceMappingURL=extension.js.map
