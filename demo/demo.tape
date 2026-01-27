# VHS Demo Tape for f
# Run with: vhs demo/demo.tape

Output demo.gif
Output demo.ascii
Set FontSize 20
Set Width 900
Set Height 500
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Setup: create a temp git repo with some changes
Hide
Type "export PS1='> '"
Enter
Sleep 100ms
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

# Demo 1/6: List changed files
Type "# Demo 1/6: List changed files with stable IDs"
Enter
Sleep 500ms
Type "f"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Demo 2/6: View diff by ID
Type "# Demo 2/6: View diff for a file"
Enter
Sleep 500ms
Type "f d d"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Demo 3/6: Edit file by ID
Type "# Demo 3/6: Edit a file in $EDITOR"
Enter
Sleep 500ms
Type "export EDITOR='vi -u NONE'"
Enter
Sleep 300ms
Type "f d e"
Enter
Sleep 1s
# vim: Go to end, open line above, type, save
Type "G"
Sleep 300ms
Type "o"
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
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Demo 4/6: Stage a file using ID
Type "# Demo 4/6: Stage a file by ID"
Enter
Sleep 500ms
Type "f d a"
Enter
Sleep 1s

# Show updated status
Type "f"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Demo 5/6: Stage remaining files
Type "# Demo 5/6: Stage another file (autoselect top of list)"
Enter
Sleep 500ms
Type "f a"
Enter
Sleep 1s

Type "f"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Demo 6/6: Commit
Type "# Demo 6/6: Commit staged changes"
Enter
Sleep 500ms
Type "f c update greeting and add notes"
Enter
Sleep 1s

# Final status
Type "f"
Enter
Sleep 3s
