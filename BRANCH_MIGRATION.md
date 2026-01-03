# Branch Migration: master to main

This document outlines the process to migrate from `master` to `main` as the default branch.

## Steps to Complete the Migration

### 1. Create the main branch from master

The `main` branch needs to be created from the current `master` branch:

```bash
git checkout master
git pull origin master
git checkout -b main
git push origin main
```

Alternatively, this can be done directly on GitHub:
1. Go to the repository on GitHub
2. Click on the branch dropdown
3. Type `main` in the search box
4. Click "Create branch: main from master"

### 2. Set main as the default branch

This must be done through GitHub's web interface:

1. Go to the repository Settings
2. Click on "Branches" in the left sidebar
3. In the "Default branch" section, click the switch icon
4. Select `main` from the dropdown
5. Click "Update" and confirm the change

### 3. Update local repositories

After the migration, collaborators should update their local repositories:

```bash
git fetch origin
git checkout main
git branch -u origin/main main
```

### 4. Optional: Delete the master branch

After ensuring everything works correctly with the main branch:

```bash
git push origin --delete master
```

Or through GitHub's web interface:
1. Go to Settings > Branches
2. Find the `master` branch in the branch list
3. Click the trash icon to delete it

## Status

- [ ] Create `main` branch from `master`
- [ ] Set `main` as default branch in GitHub settings
- [ ] Update CI/CD configurations if needed
- [ ] Notify collaborators of the change
- [ ] (Optional) Delete `master` branch
