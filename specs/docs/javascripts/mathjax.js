window.MathJax = {
  loader: {
    load: [
      "[tex]/color",
      "[tex]/ams",
      "[tex]/physics",
      "[tex]/noerrors",
      "[tex]/noundefined",
      "[tex]/amscd",
    ],
  },
  tex: {
    inlineMath: [
      ["$", "$"],
      ["\\(", "\\)"],
    ],
    displayMath: [
      ["$$", "$$"],
      ["\\[", "\\]"],
    ],
    processEscapes: true,
    processEnvironments: true,
    packages: {
      "[+]": ["color", "ams", "physics", "noerrors", "noundefined", "amscd"],
    },
    macros: {
      // Logic and set theory
      eqdef: "\\triangleq",
      iff: "\\Leftrightarrow",
      implies: "\\Rightarrow",

      // Category theory
      cat: ["{\\mathbf{#1}}", 1],
      Set: "\\mathbf{Set}",
      Cat: "\\mathbf{Cat}",
      id: "\\mathrm{id}",
      comp: "\\circ",

      // Type theory
      type: "\\mathcal{U}",
      lam: "\\lambda",
      app: "\\,",

      // Common mathematical operators
      N: "\\mathbb{N}",
      Z: "\\mathbb{Z}",
      Q: "\\mathbb{Q}",
      R: "\\mathbb{R}",
      C: "\\mathbb{C}",

      // Brackets and delimiters
      p: ["{\\left( #1 \\right)}", 1],
      cb: ["{\\left\\{ #1 \\right\\}}", 1],
      sqb: ["{\\left[ #1 \\right]}", 1],
      abs: ["{\\left| #1 \\right|}", 1],
      an: ["{\\left\\langle #1 \\right\\rangle}", 1],
      ceil: ["{\\left\\lceil #1 \\right\\rceil}", 1],
      floor: ["{\\left\\lfloor #1 \\right\\rfloor}", 1],

      // Text formatting
      tb: ["{\\textbf{#1}}", 1],
      ti: ["{\\emph{#1}}", 1],
      tt: ["{\\texttt{#1}}", 1],

      // Colors for emphasis
      red: ["{\\color{red}{#1}}", 1],
      blue: ["{\\color{blue}{#1}}", 1],
      green: ["{\\color{green}{#1}}", 1],
      gray: ["{\\color{gray}{#1}}", 1],
    },
  },
  options: {
    ignoreHtmlClass: ".*|",
    processHtmlClass: "arithmatex",
  },
  svg: {
    scale: 1,
    minScale: 0.5,
    mtextInheritFont: true,
    merrorInheritFont: true,
    mathmlSpacing: false,
    skipAttributes: {},
    exFactor: 0.5,
    displayAlign: "center",
    displayIndent: "0",
    fontCache: "local",
    localID: null,
    internalSpeechTitles: true,
    titleID: 0,
  },
};

document$.subscribe(() => {
  MathJax.typesetPromise().catch((e) => console.log(e.message));
});
