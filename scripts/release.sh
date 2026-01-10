#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 0.1.0"
  exit 1
fi

# Remove 'v' prefix if provided
VERSION=${VERSION#v}

TAG="v$VERSION"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: Tag $TAG already exists"
  exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
  echo "Error: You have uncommitted changes. Commit or stash them first."
  exit 1
fi

echo "Updating version to $VERSION..."

# Update version in tauri.conf.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Update version in package.json
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" package.json

# Update version in Cargo.toml
sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Commit version bump
git add src-tauri/tauri.conf.json package.json src-tauri/Cargo.toml
git commit -m "Bump version to $VERSION"

echo "Creating release $TAG..."
git tag "$TAG"
git push origin main "$TAG"

echo ""
echo "Release $TAG triggered!"
echo "Watch the build at: https://github.com/becked/kithara/actions"
echo "Release will appear at: https://github.com/becked/kithara/releases/tag/$TAG"
