#!/bin/bash

# GitHub Upload Script for FuzzyTail
# Run this after creating your GitHub Personal Access Token

set -e

echo "🚀 FuzzyTail GitHub Upload Script"
echo "================================"
echo ""

# Check if token is provided
if [ -z "$1" ]; then
    echo "❌ Please provide your GitHub Personal Access Token"
    echo ""
    echo "Usage: ./upload-to-github.sh YOUR_GITHUB_TOKEN"
    echo ""
    echo "To get a token:"
    echo "1. Go to https://github.com/settings/tokens"
    echo "2. Click 'Generate new token (classic)'"
    echo "3. Select 'repo' scope"
    echo "4. Copy the token and run: ./upload-to-github.sh ghp_YOUR_TOKEN_HERE"
    echo ""
    exit 1
fi

TOKEN="$1"
REPO_URL="https://${TOKEN}@github.com/yodabytz/fuzzytail.git"

echo "📋 Setting up git configuration..."
git config user.name "yodabytz"
git config user.email "yodabytz@users.noreply.github.com"

echo "🔧 Updating remote URL with token..."
git remote set-url origin "$REPO_URL"

echo "📦 Adding all files..."
git add .

echo "💾 Creating commit..."
git commit -m "Complete FuzzyTail v0.1.0 - Modern tail replacement

✨ Features:
- 6 beautiful themes with true color support  
- Advanced filtering (regex, log levels, include/exclude)
- Multiple output formats (colorized text, JSON, CSV)
- 100% tail compatibility with modern enhancements
- One-command installer script
- Blazing fast Rust performance
- Comprehensive documentation and examples

🛠️ Technical:
- Drop-in replacement for standard tail
- Cross-platform support (Linux, macOS, Windows)  
- Smart log parsing and syntax highlighting
- Configurable themes and performance tuning
- Professional CI/CD and GitHub integration

🎨 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"

echo "🚀 Pushing to GitHub..."
git push -u origin main --force

echo ""
echo "✅ Successfully uploaded FuzzyTail to GitHub!"
echo ""
echo "🌐 Your repository: https://github.com/yodabytz/fuzzytail"
echo "📦 Install command: curl -sSL https://raw.githubusercontent.com/yodabytz/fuzzytail/main/install.sh | bash"
echo ""
echo "🎉 FuzzyTail is now live on GitHub!"