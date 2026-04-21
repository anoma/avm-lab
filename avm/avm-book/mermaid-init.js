document.addEventListener('DOMContentLoaded', function() {
    const script = document.createElement('script');
    script.type = 'module';
    script.textContent = `import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs'; mermaid.initialize({startOnLoad: true, theme: 'default'});`;
    document.head.appendChild(script);
});
