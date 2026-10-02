#!/bin/sh
# Publishes the browser version to GitHub Pages: puts the contents of
# web/dist on the gh-pages branch of the "origin" remote, which GitHub
# serves as a website.
#
# Run tools/build_web.sh first, then:  tools/publish_web.sh
#
# The gh-pages branch holds only the latest build. Each publish replaces it
# (a forced push), so old builds do not pile up in the repository. Nothing
# else is ever kept on that branch.
set -e
cd "$(dirname "$0")/.."

if [ ! -f web/dist/index.html ]; then
    echo "web/dist is missing: run tools/build_web.sh first" >&2
    exit 1
fi

remote=$(git remote get-url origin)
source_commit=$(git rev-parse --short HEAD)

# Build the one-commit branch in a throwaway folder, removed again at the end.
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp -R web/dist/. "$work"
# Tells GitHub to serve the files exactly as they are.
touch "$work/.nojekyll"

cd "$work"
git init --quiet --initial-branch=gh-pages
git add --all
git commit --quiet --message "Web build of $source_commit"
git push --force "$remote" gh-pages

echo "Published. GitHub takes a minute or two to update the site."
