#!/bin/bash

# FuzzyTail GitHub Upload Script (SSH)
# Make sure your SSH key is added to your GitHub account first!

set -e

echo "🚀 FuzzyTail GitHub Upload Script (SSH)"
echo "====================================="
echo ""

# Check if SSH key exists
if [ ! -f ~/.ssh/gitid.rsa ]; then
    echo "❌ SSH key ~/.ssh/gitid.rsa not found!"
    exit 1
fi

echo "🔑 Using SSH key: ~/.ssh/gitid.rsa"
echo "📧 Key email: yodabytz@gmail.com"
echo ""

echo "⚠️  IMPORTANT: Make sure this SSH key is added to your GitHub account!"
echo "   1. Go to: https://github.com/settings/keys"
echo "   2. Click 'New SSH key'"  
echo "   3. Paste this public key:"
echo ""
cat ~/.ssh/gitid.rsa.pub
echo ""
echo "   4. Save the key, then run this script again"
echo ""

read -p "🔑 Is your SSH key added to GitHub? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Please add your SSH key to GitHub first, then run this script again."
    exit 1
fi

echo "📋 Setting up git configuration..."
git config user.name "yodabytz"
git config user.email "yodabytz@gmail.com"

echo "🔧 Setting remote to use SSH..."
git remote set-url origin git@github.com:yodabytz/fuzzytail.git

echo "🧪 Testing SSH connection to GitHub..."
if ssh -T git@github.com -o StrictHostKeyChecking=no 2>&1 | grep -q "successfully authenticated"; then
    echo "✅ SSH connection successful!"
else
    echo "❌ SSH connection failed. Please check:"
    echo "   - SSH key is added to your GitHub account"
    echo "   - You have access to yodabytz/fuzzytail repository"
    exit 1
fi

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