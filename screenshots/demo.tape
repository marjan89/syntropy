# Syntropy README Demo
# Record with: vhs demo.tape
#
# Produces: demo.gif

Output demo.gif

Set Shell "zsh"
Set FontFamily "MesloLGS Nerd Font Propo"
Set FontSize 16
Set Width 1800
Set Height 800
Set Padding 20
Set TypingSpeed 60ms
Set Theme "Catppuccin Mocha"

# Launch syntropy
Type "syntropy"
Sleep 300ms
Enter
Sleep 1500ms

# --- PART 1: Create a note ---

# Fuzzy search for notes plugin
Type "not"
Sleep 800ms

# Enter notes plugin
Enter
Sleep 800ms

# Select "New Note" task (should be second: delete, new, notes)
Down
Sleep 300ms
Enter
Sleep 1000ms

# nvim opens - type a note
Type "iHello from syntropy!"
Sleep 500ms

# Save and quit nvim
Escape
Sleep 300ms
Type ":wq"
Enter
Sleep 1000ms

# Back in syntropy - now go to "Notes" task to see our note
# We should be back at task list, navigate to "notes" task
Down
Sleep 300ms
Enter
Sleep 1s

# Navigate to the newly created note (first item might be older)
Down
Sleep 300ms

# Preview shows the note content - linger to show it
Sleep 2s

# Back out to plugin list
Escape
Sleep 300ms
Escape
Sleep 300ms

# --- PART 2: System monitoring with live graphs ---

# Clear search and find left plugin
Backspace
Backspace
Backspace
Sleep 300ms
Type "left"
Sleep 800ms

# Enter left plugin
Enter
Sleep 800ms

# Navigate to "monitor" task (3rd from top: list, list-user, monitor)
Down
Down
Sleep 300ms
Enter
Sleep 1500ms

# Search for WindowServer - always has good graphs
Type "window"
Sleep 1s

# Linger on preview with live graphs
Sleep 4s

# Back out to plugin list
Escape
Sleep 300ms
Escape
Sleep 300ms

# --- PART 3: HELLO animation ---

# Clear search and find demo plugin
Backspace
Backspace
Backspace
Backspace
Sleep 300ms
Type "dem"
Sleep 800ms

# Enter demo plugin
Enter
Sleep 800ms

# Select "animate" task (first one)
Enter
Sleep 500ms

# Watch the HELLO animation loop
Sleep 5s

# Exit cleanly
Escape
Sleep 300ms
Escape
Sleep 300ms
Escape
Sleep 300ms
