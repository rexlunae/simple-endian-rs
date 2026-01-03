#!/bin/bash
# Script to migrate from master to main branch
# This script should be run by a repository maintainer with push access

set -euo pipefail

echo "======================================================"
echo "Branch Migration Script: master -> main"
echo "======================================================"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Check if on master branch
current_branch=$(git branch --show-current)
if [ "$current_branch" != "master" ]; then
    echo "Warning: Not on master branch (current: $current_branch)"
    read -p "Do you want to checkout master? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout master
        git pull origin master
    else
        echo "Aborted."
        exit 1
    fi
fi

# Check if main branch already exists locally
if git show-ref --verify --quiet refs/heads/main; then
    echo "Warning: Local main branch already exists"
    read -p "Do you want to delete and recreate it? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git branch -D main
    else
        echo "Aborted."
        exit 1
    fi
fi

# Check if main branch exists on remote
if git ls-remote --heads origin main | grep -q main; then
    echo "Warning: Remote main branch already exists"
    echo "This script will not delete or modify the existing remote main branch."
    echo "Please resolve this manually."
    exit 1
fi

# Create main branch from master
echo ""
echo "Step 1: Creating main branch from master..."
git checkout -b main

# Push main branch to remote
echo ""
echo "Step 2: Pushing main branch to remote..."
git push -u origin main

echo ""
echo "✅ Main branch created and pushed successfully!"
echo ""
echo "======================================================"
echo "NEXT STEPS (Manual action required):"
echo "======================================================"
echo ""

# Try to extract repo URL for convenience
REPO_URL=$(git remote get-url origin 2>/dev/null || echo "")
if [[ -n "$REPO_URL" ]]; then
    # Remove .git suffix if present
    REPO_URL="${REPO_URL%.git}"
    # Convert SSH to HTTPS format
    REPO_URL="${REPO_URL/git@github.com:/https:\/\/github.com\/}"
    echo "1. Go to your GitHub repository Settings"
    echo "   URL: ${REPO_URL}/settings"
else
    echo "1. Go to your GitHub repository Settings page"
fi
echo ""
echo "2. Click on 'Branches' in the left sidebar"
echo ""
echo "3. In the 'Default branch' section:"
echo "   - Click the switch/pencil icon"
echo "   - Select 'main' from the dropdown"
echo "   - Click 'Update' and confirm"
echo ""
echo "4. (Optional) Update branch protection rules for main"
echo ""
echo "5. (Optional) After verifying everything works, delete master:"
echo "   git push origin --delete master"
echo ""
echo "6. Notify collaborators to update their local repos:"
echo "   git fetch origin"
echo "   git checkout main"
echo "   git branch -u origin/main main"
echo ""
echo "======================================================"
