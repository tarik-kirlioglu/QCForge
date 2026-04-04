# QCForge Demo Recording
# Prerequisites: cargo build
# Run: vhs demo.tape

Output demo.gif
Set FontSize 14
Set Width 1400
Set Height 800
Set TypingSpeed 75ms

# Pre-build and put binary on PATH
Hide
Type "cargo build 2>&1 && export PATH=$PWD/target/debug:$PATH && echo READY"
Enter
Sleep 120s
Type "clear"
Enter
Sleep 1s
Show

Sleep 500ms

# Launch QCForge with demo data
Type "qcforge demo_data/"
Enter

# ── Splash Screen ── DNA helix animation (~6.4s)
Sleep 9s

# ── Summary Tab ── threshold-colored metrics
Sleep 1s

# Navigate rows
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "k"
Sleep 400ms
Type "k"
Sleep 1s

# Horizontal scroll to reveal more metrics
Type "l"
Sleep 500ms
Type "l"
Sleep 500ms
Type "l"
Sleep 500ms
Type "l"
Sleep 1.5s

# Scroll back
Type "h"
Sleep 500ms
Type "h"
Sleep 500ms
Type "h"
Sleep 500ms
Type "h"
Sleep 1s

# Sort by different columns
Type "s"
Sleep 500ms
Type "s"
Sleep 500ms
Type "s"
Sleep 1s

# Reverse sort direction
Type "S"
Sleep 1.5s

# ── Overview Tab ── sortable + filterable table
Right
Sleep 2s

# Sort columns
Type "s"
Sleep 500ms
Type "s"
Sleep 1.5s

# Search for tumor samples
Type "/"
Sleep 400ms
Type "tumor"
Sleep 1s
Enter
Sleep 1.5s

# Clear filter
Type "/"
Sleep 300ms
Escape
Sleep 1s

# ── samtools Tab ── per-file alignment stats
Right
Sleep 2s

# Cycle through samples
Type "n"
Sleep 1.5s
Type "n"
Sleep 1.5s
Type "p"
Sleep 1.5s

# ── bcftools Tab ── variant call stats
Right
Sleep 2s

# Navigate variant files
Type "n"
Sleep 1.5s
Type "n"
Sleep 1.5s

# ── FastQC Tab ── read quality modules
Right
Sleep 2s

# Cycle through FastQC reports
Type "n"
Sleep 1.5s
Type "n"
Sleep 1.5s

# Show help overlay
Type "?"
Sleep 2.5s
Type "?"
Sleep 1s

# Quit
Type "q"
Sleep 500ms
