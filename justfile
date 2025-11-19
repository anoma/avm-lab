default:
    @just --list

specs-serve:
    #!/usr/bin/env bash
    cd specs
    uv run agda-mkdocs serve --config mkdocs.yml

specs-build:
    #!/usr/bin/env bash
    cd specs
    # Copy assets to docs directory
    cp -r assets/stylesheets docs/
    cp -r assets/javascripts docs/
    cp -r assets/assets docs/
    export SITE_URL=https://anoma.github.io/avm-lab/
    export SITE_DIR=build/site
    uv run agda-mkdocs build --config mkdocs.yml

specs-install:
    cd specs && uv sync

specs-clean:
    #!/usr/bin/env bash
    rm -rf specs/build/site
    cd specs && echo "yes" | uv run agda-mkdocs clear-cache --all

specs-test:
    #!/usr/bin/env bash
    cd specs/docs
    agda everything.lagda.md
