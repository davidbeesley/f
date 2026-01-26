# VHS Demo Tape for f
# Run with: vhs demo/demo.tape

Output demo/demo.gif
Set FontSize 16
Set Width 900
Set Height 500
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms

# Setup: create a temp git repo with some changes
Hide
Type "cd $(mktemp -d)"
Enter
Sleep 100ms
Type "git init -q && git config user.email 'demo@example.com' && git config user.name 'Demo'"
Enter
Sleep 100ms
Type "echo 'hello' > greeting.txt"
Enter
Type "git add . && git commit -q -m 'init'"
Enter
Sleep 100ms
Type "echo 'hello world' > greeting.txt"
Enter
Type "echo 'new file' > notes.txt"
Enter
Type "echo 'config' > config.toml"
Enter
Type "git add config.toml"
Enter
Sleep 100ms
Type "clear"
Enter
Sleep 300ms
Show

# Demo: List changed files
Type "# List changed files with stable IDs"
Enter
Sleep 500ms
Type "f"
Enter
Sleep 2s

# Demo: View diff by ID
Type "# View diff for a file"
Enter
Sleep 500ms
Type "f d d"
Enter
Sleep 2s
Type "q"
Sleep 500ms

# Demo: Edit file by ID
Type "# Edit a file in $EDITOR"
Enter
Sleep 500ms
Type "export EDITOR=vi"
Enter
Sleep 300ms
Type "f d e"
Enter
Sleep 1s
# vim: Go to end, open line above, type, save
Type "G"
Sleep 300ms
Type "O"
Sleep 300ms
Type "some changes here"
Sleep 500ms
Escape
Sleep 300ms
Type ":wq"
Enter
Sleep 1s

# Show status after edit
Type "f"
Enter
Sleep 2s

# Demo: Stage a file using ID
Type "# Stage a file by ID"
Enter
Sleep 500ms
Type "f d a"
Enter
Sleep 1s

# Show updated status
Type "f"
Enter
Sleep 2s

# Demo: Stage remaining files
Type "# Stage another file (autoselect top of list)"
Enter
Sleep 500ms
Type "f a"
Enter
Sleep 1s

Type "f"
Enter
Sleep 2s

# Demo: Commit
Type "# Commit staged changes"
Enter
Sleep 500ms
Type "f c update greeting and add notes"
Enter
Sleep 1s

# Final status
Type "f"
Enter
Sleep 2s
