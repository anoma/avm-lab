#!/usr/bin/env bash
# Build script for AVM slides
# Uses XeLaTeX for PragmataPro Mono font support

set -e

# Change to script directory
cd "$(dirname "$0")"

MAIN="main"
OUTPUT_DIR="."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building AVM slides with XeLaTeX...${NC}"

# First pass
echo -e "${YELLOW}→ XeLaTeX pass 1/3${NC}"
xelatex -interaction=nonstopmode -output-directory="$OUTPUT_DIR" "$MAIN.tex" > /dev/null 2>&1 || true

# BibTeX (may fail due to permissions, that's ok)
echo -e "${YELLOW}→ BibTeX${NC}"
bibtex "$MAIN" 2>/dev/null || echo -e "${YELLOW}  (bibtex skipped - permissions issue, continuing...)${NC}"

# Second pass
echo -e "${YELLOW}→ XeLaTeX pass 2/3${NC}"
xelatex -interaction=nonstopmode -output-directory="$OUTPUT_DIR" "$MAIN.tex" > /dev/null 2>&1 || true

# Third pass
echo -e "${YELLOW}→ XeLaTeX pass 3/3${NC}"
xelatex -interaction=nonstopmode -output-directory="$OUTPUT_DIR" "$MAIN.tex" > /dev/null 2>&1 || true

if [ -f "$MAIN.pdf" ]; then
    SIZE=$(du -h "$MAIN.pdf" | cut -f1)
    PAGES=$(pdfinfo "$MAIN.pdf" 2>/dev/null | grep "Pages:" | awk '{print $2}' || echo "?")
    echo -e "${GREEN}✓ Success!${NC} Generated $MAIN.pdf ($SIZE, $PAGES pages)"

    # Check for PragmataPro
    if pdffonts "$MAIN.pdf" 2>/dev/null | grep -q "PragmataPro"; then
        echo -e "${GREEN}✓ PragmataPro Mono embedded successfully${NC}"
    else
        echo -e "${YELLOW}⚠ Warning: PragmataPro not found in PDF (using fallback font?)${NC}"
    fi
else
    echo -e "${RED}✗ Build failed - no PDF generated${NC}"
    exit 1
fi
