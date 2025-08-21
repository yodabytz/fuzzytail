# 🚀 GitHub Setup Instructions

## Step 1: Create GitHub Personal Access Token

1. **Go to GitHub**: https://github.com/settings/tokens
2. **Sign in** as `yodabytz`  
3. **Click**: "Generate new token" → "Generate new token (classic)"
4. **Fill out**:
   - **Note**: "FuzzyTail Development"
   - **Expiration**: 90 days (or your preference)
   - **Scopes**: ✅ Check **"repo"** (full repository access)
5. **Click**: "Generate token"
6. **Copy the token** (starts with `ghp_...`) - **Save it safely!**

## Step 2: Upload Using Script (Recommended)

```bash
# Run the upload script with your token
./upload-to-github.sh ghp_YOUR_TOKEN_HERE
```

## Step 3: Manual Upload (Alternative)

If the script doesn't work, run these commands manually:

```bash
# Configure git
git config user.name "yodabytz"
git config user.email "yodabytz@users.noreply.github.com"

# Set remote with token (replace ghp_YOUR_TOKEN_HERE with your actual token)
git remote set-url origin https://ghp_YOUR_TOKEN_HERE@github.com/yodabytz/fuzzytail.git

# Add all files
git add .

# Commit
git commit -m "Complete FuzzyTail v0.1.0 - Modern tail replacement

✨ Features:
- 6 beautiful themes with true color support
- Advanced filtering and multiple output formats
- 100% tail compatibility with modern enhancements
- One-command installer and comprehensive docs

🎨 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"

# Push (force push to replace existing content)
git push -u origin main --force
```

## Step 4: Verify Upload

After successful upload:
- Visit: https://github.com/yodabytz/fuzzytail  
- Test install: `curl -sSL https://raw.githubusercontent.com/yodabytz/fuzzytail/main/install.sh | bash`

## 🔐 Security Notes

- **Never share your token** - treat it like a password
- **Use token expiration** - set reasonable expiry dates
- **Delete unused tokens** - clean up old tokens regularly
- **Store securely** - use a password manager

## 🚨 If You Get Errors

**"Authentication failed"**:
- Check your token is correct (starts with `ghp_`)
- Verify token has "repo" scope
- Make sure token hasn't expired

**"Permission denied"**:
- Confirm you're the owner of yodabytz/fuzzytail repo
- Check token permissions include repository access

**"Remote already exists"**:
- This is normal, the script updates the existing remote

## 🎉 Success!

Once uploaded, your FuzzyTail will be available at:
- **Repository**: https://github.com/yodabytz/fuzzytail
- **One-line install**: `curl -sSL https://raw.githubusercontent.com/yodabytz/fuzzytail/main/install.sh | bash`
- **Clone command**: `git clone https://github.com/yodabytz/fuzzytail.git`