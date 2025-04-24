# `confsync` Command Architecture  

---

## **Main Commands**  

### **1. `init`**  
*Initialize a new backup repository (local or remote).*  
```bash  
confsync init [REPO_URL] [FLAGS]  
```  
| Argument   | Description                          |  
|------------|--------------------------------------|  
| `remote` | (Optional) Git remote URL (SSH/HTTPS)|  

**Flags**:  
- `--git`: Initialize git repo at `~/.local/share/confsync` [default]
- `--force`: Overwrite existing configuration  

---

### **2. `add`**  
*Track a file/directory for backup.*  
```bash  
confsync add <PATH> [--alias NAME] [FLAGS]  
```  
| Argument | Description                  |  
|----------|------------------------------|  
| `PATH`   | Absolute or relative path    |  

**Flags**:  
- `--alias`: Human-readable name (e.g., `zsh` for `~/.zshrc`)  
- `--encrypt`: Mark file for encryption (Phase 2)  

---

### **3. `backup`**  
*Commit changes and push to repo.*  
```bash  
confsync backup [FLAGS]  
```  
**Flags**:  
- `--message "-m"`: Custom commit message (default: "Backup: <timestamp>")  
- `--force`: Push even if no changes detected  
- `--dry-run`: Show preview without committing  

---

### **4. `restore`**  
*Restore files from a backup.*  
```bash  
confsync restore [TARGET] [FLAGS]  
```  
| Argument | Description                          |  
|----------|--------------------------------------|  
| `TARGET` | (Optional) Commit hash/tag (e.g., `@latest`)|  

**Flags**:  
- `--dry-run`: Show files to restore without modifying disk  
- `--force`: Overwrite local changes  

---

### **5. `list`**  
*Show backup history.*  
```bash  
confsync list [FLAGS]  
```  
**Flags**:  
- `--alias`: Shows history for an alias  
- `--verbose`: Show changed files/aliases  

---

## **Advanced Commands (Phase 2+)**  

### **6. `watch`**  
*Daemon mode for auto-backup on file changes.*  
```bash  
confsync watch [FLAGS]  
```  
**Flags**:  
- `--debounce`: Delay (ms) before triggering backup (default: 2000)  

---

### **7. `profile`**  
*Manage multiple backup profiles.*  
```bash  
confsync profile <SUBCOMMAND>  
```  
**Subcommands**:  
- `create <NAME> [REPO_URL]`: New profile  
- `list`: Show all profiles  
- `switch <NAME>`: Change active profile  
- `delete <NAME>`: Remove profile (--force to skip confirmation)  

---

### **8. `encrypt`**  
*Manage encryption keys (Phase 2).*  
```bash  
confsync encrypt <SUBCOMMAND>  
```  
**Subcommands**:  
- `init`: Generate age keypair  
- `add-key <PUBKEY_PATH>`: Add public key for sharing  
- `rotate`: Generate new keys  

---

### **9. `config`**  
*Edit/view configuration.*  
```bash  
confsync config <SUBCOMMAND>  
```  
**Subcommands**:  
- `edit`: Open config file in `$EDITOR`  
- `validate`: Check for errors  
- `path`: Show config file location  

---

## **Global Flags**  
*(Available for all commands)*  
- `--verbose`: Show debug logs  
- `--paths`: Shows paths used by confsync
- `--config-path <PATH>`: Use custom config file  [phase 2]
- `--profile <NAME>`: Override active profile  

---

## **Utility Commands**  
```bash  
confsync status   # Show changed/untracked files  
confsync version  # Print version  
confsync help     # Show full help  
```

---

## **Example Workflows**  
1. **Basic Setup**:  
   ```bash  
   confsync init git@github.com:user/backups.git  
   confsync add ~/.zshrc --alias zsh  
   confsync backup  
   ```  

2. **Restore Latest**:  
   ```bash  
   confsync restore @latest  
   ```  

3. **Multi-Profile**:  
   ```bash  
   confsync profile create work git@github.com:user/work-config.git  
   confsync --profile work add ~/.work-env  
   ```  

---

## **Design Notes**  
1. **Config Storage**:  
   - Default: `~/.config/confsync/config.toml`  
   - Tracked files stored in-place (no separate directory)  

2. **Git Behavior**:  
   - Always uses `main` branch  
   - Auto-generated `.gitignore` excludes binary/large files  

3. **Alias Resolution**:  
   - Files identified by alias (e.g., `zsh`) or path in `list`/`restore`  

4. **Security**:  
   - Never store credentials—rely on SSH agent or system keyring  
   - Encrypted files stored as `.age` extensions in Git (Phase 2)  
