import React, { useState } from 'react';
import { BoltIcon, DocumentTextIcon, CodeBracketIcon, ServerStackIcon, PuzzlePieceIcon } from '@heroicons/react/24/outline';

const PAGES = [
  { id: 'intro', title: 'Introduction', icon: DocumentTextIcon },
  { id: 'getting-started', title: 'Getting Started', icon: BoltIcon },
  { id: 'control-plane', title: 'Control Plane (IDE)', icon: ServerStackIcon },
  { id: 'framework-adapters', title: 'Framework Integration', icon: PuzzlePieceIcon },
  { id: 'multi-function', title: 'Multi-Function Project', icon: CodeBracketIcon },
  { id: 'case-studies', title: 'Case Studies', icon: CodeBracketIcon },
];

const Intro = () => (
  <div>
    <h1>Wednes Serverless Engine</h1>
    <p>Welcome to the official documentation for <strong>Wednes Engine</strong>, an ultra-fast, WebAssembly Component Model-based serverless runtime designed for Web, AI, and Microservices workloads.</p>
    
    <h2>What makes it different?</h2>
    <p>Wednes Engine relies on the <code>wasi</code> Component Model and Wasmtime to offer instant execution capabilities. Instead of booting up containers (Docker) or allocating large memory blocks for idle servers, Wednes Engine executes WebAssembly modules with a cold start of <code>&lt; 5ms</code>.</p>
    
    <ul>
      <li><strong>Blazing Fast Cold Starts</strong>: Eliminates the typical serverless cold start penalty.</li>
      <li><strong>Polyglot</strong>: Write in Rust, Go, TypeScript/JS, or Python.</li>
      <li><strong>Framework Compatibility</strong>: Bring your existing Web Frameworks (Axum, Fiber, Express, FastAPI) by bridging requests.</li>
      <li><strong>Microservices Ready</strong>: Supports the AWS SAM-like <code>wednes.yaml</code> manifest pattern to manage multi-function workspaces.</li>
    </ul>

    <h2>The Control Plane</h2>
    <p>You don't need a heavy local setup. We offer an integrated Web IDE at <code>cp.wednesdev.id</code> where you can write, deploy, and execute WebAssembly functions straight from the browser.</p>
  </div>
);

const GettingStarted = () => (
  <div>
    <h1>Getting Started</h1>
    <p>Getting started with Wednes Engine is simple. You can either use our cloud Control Plane or download the SDK and run your functions locally.</p>

    <h2>Using the Control Plane (Recommended)</h2>
    <ol className="list-decimal ml-6 space-y-2 mb-6">
      <li>Open <a href="https://cp.wednesdev.id" target="_blank" rel="noreferrer">cp.wednesdev.id</a>.</li>
      <li>Click <strong>+ New</strong> on the Explorer sidebar.</li>
      <li>Choose your preferred language: Rust, Go, TypeScript, or Python.</li>
      <li>Hit <strong>Create & Open in IDE</strong>.</li>
      <li>Edit your code in the Monaco Editor.</li>
      <li>Click the <strong>Run Function (Ctrl+Enter)</strong> CodeLens or hit the <strong>Deploy</strong> button.</li>
      <li>Your function is immediately live and testable!</li>
    </ol>

    <h2>Invoking Your Function</h2>
    <p>Once deployed, your function is accessible via our Edge Gateway:</p>
    <pre><code>{`curl -X POST https://cp.wednesdev.id/api/run/project-name-function-name \\
  -H "Content-Type: application/json" \\
  -d '{"message": "Hello"}'`}</code></pre>
  </div>
);

const ControlPlane = () => (
  <div>
    <h1>Control Plane (IDE)</h1>
    <p>The Wednesdev Control Plane (<code>cp.wednesdev.id</code>) acts as your central hub for authoring and deploying Wasmtime components.</p>

    <h2>1. Zero-Setup Environment</h2>
    <p>All compilation happens on our internal KVM nodes. You do not need to install cargo, rustc, node, tsc, go, or componentize-js on your computer.</p>

    <h2>2. Auto-Save & Persistent State</h2>
    <p>You never lose your code. As you type, the Web IDE debounces and autosaves your progress directly to our PostgreSQL database. If you refresh your browser, your code is right there.</p>

    <h2>3. Real-time Deployment & Feedback</h2>
    <p>When you click <strong>Deploy</strong>, the UI streams the build progress from the KVM builder. Thanks to optimizations like <code>sccache</code> and bypass mechanisms in our componentizers, typical builds take 1–3 seconds, allowing for a tight, immediate feedback loop.</p>

    <h2>4. Multi-Function Support</h2>
    <p>The IDE's Explorer inherently understands <code>wednes.yaml</code>. Clicking <em>Deploy</em> on a project containing multiple handler files automatically builds multi-artifacts and pushes all of them to the active registry simultaneously.</p>

    <h2>5. Project Export</h2>
    <p>You can always download your source code + Wasm Manifest using the <strong>Download (.zip)</strong> button in the Explorer panel.</p>
  </div>
);

const FrameworkAdapters = () => (
  <div>
    <h1>Framework Integration</h1>
    <p>The Wednes Engine exposes the <code>wednes:function@0.1.0</code> ABI:</p>
    <pre><code>{`interface types {
  record header { name: string, value: string }
  record request { method: string, path: string, headers: list<header>, body: list<u8> }
  record response { status: u16, headers: list<header>, body: list<u8> }
}

world function {
  use types.{request, response};
  export handle: func(req: request) -> response;
}`}</code></pre>
    
    <p>To use familiar frameworks like Express, FastAPI, or Axum, you simply use an adapter pattern to bridge this ABI to your framework's native objects.</p>

    <h2>Python (FastAPI / ASGI Adapter)</h2>
    <p>When writing Python, parse the incoming <code>Request</code> and pass it to your ASGI app.</p>
    <pre><code>{`from wit_world.imports.types import Request, Response, Header
import json

class WitWorld:
    def handle(self, req: Request) -> Response:
        if req.path == "/webhook/shopify" and req.method == "POST":
            return Response(status=200, headers=[], body=b'{"status": "received"}')
        return Response(status=404, headers=[], body=b'Not Found')`}</code></pre>

    <h2>TypeScript (Express Adapter)</h2>
    <pre><code>{`export function handle(req: any) {
  if (req.path === "/webhook/whatsapp" && req.method === "POST") {
    return {
      status: 200,
      headers: [{ name: "content-type", value: "application/json" }],
      body: Array.from(new TextEncoder().encode(JSON.stringify({ status: "ok" })))
    };
  }
  return { status: 404, headers: [], body: [] };
}`}</code></pre>
  </div>
);

const MultiFunction = () => (
  <div>
    <h1>Multi-Function Projects</h1>
    <p>Just like AWS Serverless Application Model (SAM) or the Serverless Framework, Wednes Engine supports building multiple functions within a single repository or project using <code>wednes.yaml</code>.</p>

    <h2>The <code>wednes.yaml</code> Specification</h2>
    <p>Add a <code>wednes.yaml</code> file to your project root:</p>
    <pre><code>{`version: "1.0"
name: "crm-service"

functions:
  whatsapp:
    handler: src/whatsapp.ts
    route: /webhook/whatsapp
    lang: typescript

  shopify:
    handler: src/shopify.ts
    route: /webhook/shopify
    lang: typescript

  scraper:
    handler: src/scraper.py
    route: /api/scrape
    lang: python`}</code></pre>

    <h2>How It Works</h2>
    <ol className="list-decimal ml-6 space-y-2">
      <li><strong>Compilation</strong>: When you click <strong>Deploy</strong>, the daemon reads <code>wednes.yaml</code> and isolates every entry under functions.</li>
      <li><strong>Artifact Generation</strong>: The builder runs appropriate compilers for each target, spitting out separate WebAssembly files (e.g. <code>crm-service-whatsapp.wasm</code>).</li>
      <li><strong>Atomic Registration</strong>: All resulting <code>.wasm</code> artifacts are pushed to the engine registry simultaneously.</li>
    </ol>
  </div>
);

const CaseStudies = () => (
  <div>
    <h1>Case Studies</h1>
    <p>Here are ready-to-copy examples for real-world scenarios.</p>

    <h2>1. WhatsApp CRM Webhook (TypeScript)</h2>
    <pre><code>{`export function handle(req: any) {
  const method = req.method;
  
  if (method === "GET") {
    return {
      status: 200,
      headers: [{ name: "content-type", value: "text/plain" }],
      body: Array.from(new TextEncoder().encode("CHALLENGE_ACCEPTED"))
    };
  }

  if (method === "POST") {
    const bodyStr = new TextDecoder().decode(new Uint8Array(req.body));
    const data = JSON.parse(bodyStr);
    const sender = data.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.from || "unknown";

    const reply = {
      messaging_product: "whatsapp",
      to: sender,
      text: { body: "Halo! Tim CRM Wednesdev akan segera menghubungi Anda." }
    };

    return {
      status: 200,
      headers: [{ name: "content-type", value: "application/json" }],
      body: Array.from(new TextEncoder().encode(JSON.stringify(reply)))
    };
  }
  return { status: 405, headers: [], body: [] };
}`}</code></pre>

    <h2>2. Scraping & Shopify Webhook (Python)</h2>
    <pre><code>{`from wit_world.imports.types import Request, Response, Header
import json
import urllib.request

class WitWorld:
    def handle(self, req: Request) -> Response:
        if req.path == "/api/scrape":
            req_obj = urllib.request.Request("https://dummyjson.com/products/1")
            with urllib.request.urlopen(req_obj) as resp:
                data = resp.read()
            return Response(status=200, headers=[Header(name="content-type", value="application/json")], body=data)

        elif req.path == "/webhook/shopify" and req.method == "POST":
            return Response(status=200, headers=[], body=b'{"acknowledged": true}')

        return Response(status=404, headers=[], body=b'{"error": "Not Found"}')`}</code></pre>
  </div>
);

export default function App() {
  const [activePage, setActivePage] = useState('intro');

  const renderContent = () => {
    switch (activePage) {
      case 'intro': return <Intro />;
      case 'getting-started': return <GettingStarted />;
      case 'control-plane': return <ControlPlane />;
      case 'framework-adapters': return <FrameworkAdapters />;
      case 'multi-function': return <MultiFunction />;
      case 'case-studies': return <CaseStudies />;
      default: return <Intro />;
    }
  };

  return (
    <div className="flex h-screen bg-[#0d1117] text-[#c9d1d9] font-sans">
      {/* Sidebar */}
      <aside className="w-64 border-r border-[#30363d] bg-[#0d1117] flex flex-col shrink-0">
        <div className="p-4 border-b border-[#30363d] flex items-center gap-2">
          <BoltIcon className="w-6 h-6 text-blue-500" />
          <span className="font-bold text-white text-lg tracking-tight">Wednes Engine</span>
        </div>
        <nav className="flex-1 p-3 space-y-1 overflow-y-auto">
          {PAGES.map(page => {
            const Icon = page.icon;
            const isActive = activePage === page.id;
            return (
              <button
                key={page.id}
                onClick={() => setActivePage(page.id)}
                className={`w-full flex items-center gap-2.5 px-3 py-2 text-sm rounded-md transition-colors ${
                  isActive ? 'bg-[#1f6feb26] text-[#58a6ff] font-medium' : 'text-[#8b949e] hover:bg-[#161b22] hover:text-[#c9d1d9]'
                }`}
              >
                <Icon className="w-4 h-4" />
                {page.title}
              </button>
            )
          })}
        </nav>
        <div className="p-4 border-t border-[#30363d] text-xs text-[#8b949e]">
          © 2026 Wednesdev
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto bg-[#0d1117]">
        <div className="max-w-4xl mx-auto px-8 py-12">
          {renderContent()}
        </div>
      </main>
    </div>
  );
}
