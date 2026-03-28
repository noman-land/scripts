#!/usr/bin/env bash
# install.sh – install available utilities as commands in $PATH with a nice TUI

set -euo pipefail

# Default install location (user-writable, commonly in PATH)
INSTALL_DIR="${1:-$HOME/.local/bin}"

# Helper functions for colored output
info()  { printf "[\033[1;34mINFO\033[0m] %s\n" "$*"; }
success() { printf "[\033[1;32mSUCCESS\033[0m] %s\n" "$*"; }
warn()  { printf "[\033[1;33mWARN\033[0m] %s\n" "$*"; }
error() { printf "[\033[1;31mERROR\033[0m] %s\n" "$*" >&2; }

# Convert a path to a tilde path if it's under $HOME
tilde_path() {
    local p="$1"
    if [[ -n "$HOME" && "$p" == "$HOME"* ]]; then
        echo "~${p#$HOME}"
    else
        echo "$p"
    fi
}

# Check if a utility is already installed
is_installed() {
    local util_name="$1"
    local executable_path="$2"
    local dest="$INSTALL_DIR/$util_name"

    if [[ -L "$dest" ]]; then
        # It's a symlink, check if it points to the correct executable
        local target
        target="$(readlink "$dest")"
        if [[ "$target" == "$executable_path" ]]; then
            return 0  # True, is installed correctly
        fi
    fi
    return 1  # False, not installed or incorrectly installed
}

# Print a nice header
print_header() {
    echo ""
    echo "================================================================"
    echo "                Utility Installer for scripts"
    echo "================================================================"
    echo ""
}

# Print a section header
print_section() {
    echo ""
    echo "--- $1 ---"
    echo ""
}

# Find utility directories (non-hidden subdirs of this script's location)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UTILITIES=()
for dir in "$SCRIPT_DIR"/*/; do
    [[ -d "$dir" ]] || continue
    # Remove trailing slash from directory path
    dir="${dir%/}"
    # Get utility name from directory name
    util_name="$(basename "$dir")"
    # Skip hidden directories
    [[ "$util_name" == .* ]] && continue
    # Look for an executable file inside the directory (commonly same name as dir)
    executable=""
    # First try: $dir/$util_name
    if [[ -x "$dir/$util_name" ]]; then
        executable="$dir/$util_name"
    else
        # Fallback: any executable in the dir (excluding this install script)
        while IFS= read -r -d '' file; do
            if [[ -x "$file" && "$file" != "$SCRIPT_DIR/install.sh"* ]]; then
                executable="$file"
                break
            fi
        done < <(find "$dir" -maxdepth 1 -type f -executable -print0 2>/dev/null)
    fi
    if [[ -n "$executable" ]]; then
        UTILITIES+=("$util_name:$executable")
    fi
done

if [[ ${#UTILITIES[@]} -eq 0 ]]; then
    error "No utilities found in $SCRIPT_DIR"
    exit 1
fi

print_header
print_section "Found Utilities"

# Show utilities with their installation status
for i in "${!UTILITIES[@]}"; do
    util="${UTILITIES[$i]%%:*}"
    executable="${UTILITIES[$i]#*:}"
    if is_installed "$util" "$executable"; then
        status="[\033[1;32mINSTALLED\033[0m]"
    else
        status="[\033[1;37mNOT INSTALLED\033[0m]"
    fi
    printf "  %2d) %-20s %b\n" $((i+1)) "$util" "$status"
done
echo ""

# Prompt for selection
print_section "Selection"
echo "Enter the numbers of the utilities you wish to toggle (install if not installed, uninstall if installed)."
echo "Examples:"
echo "  '1 3'   : toggle the 1st and 3rd utilities"
echo "  'all'   : install all utilities"
echo "  <enter> : exit without making changes"
echo ""
read -rp "Your choice: " choice

# Process the choice
case "$choice" in
    [Aa][Ll][Ll])
        SELECTED=("${UTILITIES[@]}")
        ACTION="install"
        ;;
    "")   # Handle empty input (just pressing enter)
        info "No changes made. Exiting."
        exit 0
        ;;
    *)
        # Split the choice by spaces and validate
        SELECTED=()
        read -ra choices <<< "$choice"
        for num in "${choices[@]}"; do
            if [[ "$num" =~ ^[0-9]+$ ]] && [ "$num" -ge 1 ] && [ "$num" -le ${#UTILITIES[@]} ]; then
                SELECTED+=("${UTILITIES[$((num-1))]}")
            else
                warn "Ignoring invalid selection: $num"
            fi
        done
        if [ ${#SELECTED[@]} -eq 0 ]; then
            error "No valid utilities selected. Exiting."
            exit 1
        fi
        ACTION="toggle"
        ;;
esac

print_section "Selected Utilities"
for entry in "${SELECTED[@]}"; do
    util="${entry%%:*}"
    executable="${entry#*:}"
    if is_installed "$util" "$executable"; then
        status="[\033[1;32mINSTALLED\033[0m]"
        action_text="will be uninstalled"
    else
        status="[\033[1;37mNOT INSTALLED\033[0m]"
        action_text="will be installed"
    fi
    printf "  %-20s %b (%s)\n" "$util" "$status" "$action_text"
done
echo ""

# Ensure install directory exists
if [[ ! -d "$INSTALL_DIR" ]]; then
    info "Creating install directory: $(tilde_path "$INSTALL_DIR")"
    mkdir -p "$INSTALL_DIR"
fi

# Check if the directory is already in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    warn "$(tilde_path "$INSTALL_DIR") is not currently in your PATH."
    info "Add it permanently by adding the following line to your shell rc file (e.g., ~/.bashrc, ~/.zshrc):"
    info "  export PATH=\"\$INSTALL_DIR:\$PATH\""
    info "For the current session you can run: export PATH=\"$INSTALL_DIR:\$PATH\""
fi

# Process selected utilities based on action
print_section "Processing"
for entry in "${SELECTED[@]}"; do
    util_name="${entry%%:*}"
    executable_path="${entry#*:}"
    link_name="$util_name"
    dest="$INSTALL_DIR/$link_name"

    # Determine if we should install or uninstall
    should_install=false
    if [[ "$ACTION" == "install" ]]; then
        should_install=true
    elif [[ "$ACTION" == "toggle" ]]; then
        if ! is_installed "$util_name" "$executable_path"; then
            should_install=true
        fi
    fi

    if [[ "$should_install" == true ]]; then
        # Install the utility (create symlink)
        if [[ -e "$dest" || -L "$dest" ]]; then
            # Ask before overwriting
            read -rp "A file or link named '$link_name' already exists in '$(tilde_path "$INSTALL_DIR")'. Overwrite? [y/N] " answer
            if [[ ! "$answer" =~ ^[Yy]$ ]]; then
                warn "Skipping $util_name."
                continue
            fi
            rm -f "$dest"
        fi
        ln -s "$executable_path" "$dest"
        success "Installed: $link_name -> $executable_path"
    elif [[ "$ACTION" == "toggle" ]]; then
        # Uninstall the utility (remove symlink)
        if is_installed "$util_name" "$executable_path"; then
            if [[ -L "$dest" && "$(readlink "$dest")" == "$executable_path" ]]; then
                # Only remove if it's our symlink pointing to the correct target
                rm -f "$dest"
                success "Uninstalled: $link_name"
            else
                warn "Skipping $util_name: not a valid installation or points to wrong target"
            fi
        fi
    fi
done

print_section "Completion"
if [[ "$ACTION" == "install" ]]; then
    success "Installation complete."
elif [[ "$ACTION" == "toggle" ]]; then
    success "Toggle operation complete."
fi
info "You can now run the installed utilities from your shell (if $(tilde_path "$INSTALL_DIR") is in your PATH)."
if [ ${#SELECTED[@]} -gt 0 ]; then
    first_util="${SELECTED[0]%%:*}"
    info "Example usage:"
    info "  $first_util <arguments>"
fi
echo ""
echo "================================================================"
echo ""