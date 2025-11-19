# Makefile for AVM slides
# Uses XeLaTeX for PragmataPro Mono font support

.PHONY: all clean pdf view quick show distclean check-errors

# Main target
all: pdf

# Check for critical LaTeX errors in log file
check-errors:
	@if grep -q "! LaTeX Error:" main.log 2>/dev/null; then \
		echo "ERROR: LaTeX compilation errors detected!"; \
		grep "! LaTeX Error:" main.log | head -5; \
		exit 1; \
	fi
	@if grep -q "! Emergency stop" main.log 2>/dev/null; then \
		echo "ERROR: LaTeX compilation stopped due to errors!"; \
		exit 1; \
	fi

# Build PDF with XeLaTeX (required for fontspec/PragmataPro)
pdf: main.tex
	@echo "Building PDF with XeLaTeX..."
	@xelatex -interaction=nonstopmode main.tex || (echo "ERROR: First XeLaTeX pass failed!"; exit 1)
	@$(MAKE) check-errors || exit 1
	@if grep -q "\\bibliography" main.tex && ! grep -q "^%.*\\bibliography" main.tex; then \
		echo "Running BibTeX..."; \
		bibtex main 2>/dev/null || echo "BibTeX skipped (no citations or bibliography file)"; \
		xelatex -interaction=nonstopmode main.tex || (echo "ERROR: Second XeLaTeX pass failed!"; exit 1); \
		$(MAKE) check-errors || exit 1; \
	fi
	@echo "Final XeLaTeX pass..."
	@xelatex -interaction=nonstopmode main.tex || (echo "ERROR: Final XeLaTeX pass failed!"; exit 1)
	@$(MAKE) check-errors || exit 1
	@if [ ! -f main.pdf ]; then \
		echo "ERROR: PDF file was not generated!"; \
		exit 1; \
	fi
	@echo "✓ Build complete: main.pdf"
	@echo "  File size: $$(du -h main.pdf | cut -f1)"

# Quick build (single pass, no bibtex)
quick: main.tex
	@echo "Quick build (single pass)..."
	@xelatex -interaction=nonstopmode main.tex || (echo "WARNING: Quick build had errors (check main.log)"; exit 1)

# View PDF
view: main.pdf
	@echo "Opening PDF..."
	@open main.pdf

# Clean auxiliary files
clean:
	@echo "Cleaning auxiliary files..."
	@rm -f *.aux *.log *.nav *.out *.snm *.toc *.vrb *.bbl *.blg *.bcf *.run.xml *.fls *.fdb_latexmk *.synctex.gz

# Clean everything including PDF
distclean: clean
	@echo "Cleaning everything including PDF..."
	@rm -f main.pdf

# Build and view
show: pdf view
