#!/usr/bin/env bash
set -euo pipefail

# Path to the ALSA config file
ASOUND_CONF="/etc/asound.conf"

# Ensure script is run with root privileges
if [[ $EUID -ne 0 ]]; then
   echo "[-] Error: This script modifies $ASOUND_CONF and must be run with root privileges (sudo)." >&2
   exit 1
fi

if [[ ! -f "$ASOUND_CONF" ]]; then
    echo "[-] Error: $ASOUND_CONF not found." >&2
    exit 1
fi

echo "[+] Querying available ALSA sound cards..."

# Read sound cards from /proc/asound/cards
cards_raw=$(cat /proc/asound/cards)

if [[ -z "$cards_raw" ]] || ! grep -q "\[" <<< "$cards_raw"; then
    echo "[-] No ALSA cards detected." >&2
    exit 1
fi

# Parse card numbers and descriptions into arrays
declare -a card_nums=()
declare -a card_names=()

while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*([0-9]+)[[:space:]]+\[([^]]+)\][[:space:]]*:[[:space:]]*(.*)$ ]]; then
        c_num="${BASH_REMATCH[1]}"
        c_id="${BASH_REMATCH[2]}"
        c_desc="${BASH_REMATCH[3]}"
        
        card_nums+=("$c_num")
        card_names+=("$c_id ($c_desc)")
    fi
done <<< "$cards_raw"

if [[ ${#card_nums[@]} -eq 0 ]]; then
    echo "[-] Error: Failed to parse any ALSA cards from /proc/asound/cards." >&2
    exit 1
fi

# Display the interactive list
echo ""
echo "Available ALSA Audio Cards:"
echo "----------------------------------------"
for i in "${!card_nums[@]}"; do
    echo "  [$i] Card ${card_nums[$i]}: ${card_names[$i]}"
done
echo "----------------------------------------"

# Prompt user for selection
read -rp "Select card number [0-$(( ${#card_nums[@]} - 1 ))]: " selection

# Validate input
if ! [[ "$selection" =~ ^[0-9]+$ ]] || (( selection < 0 || selection >= ${#card_nums[@]} )); then
    echo "[-] Invalid selection." >&2
    exit 1
fi

SELECTED_CARD_NUM="${card_nums[$selection]}"
SELECTED_CARD_DESC="${card_names[$selection]}"

echo "[+] Selected: Card $SELECTED_CARD_NUM - $SELECTED_CARD_DESC"

# Update /etc/asound.conf between the markers using Python
python3 -c "
conf_path = '$ASOUND_CONF'
with open(conf_path, 'r') as f:
    content = f.read()

import re
pattern = r'(# BEGIN_ACTIVE_CARD\n)(.*?)(\n# END_ACTIVE_CARD)'

def update_card(match):
    block = match.group(2)
    # Replace any 'card <number>' with the selected card number
    updated_block = re.sub(r'card\s+\d+', 'card $SELECTED_CARD_NUM', block)
    return match.group(1) + updated_block + match.group(3)

new_content = re.sub(pattern, update_card, content, flags=re.DOTALL)

with open(conf_path, 'w') as f:
    f.write(new_content)
"

echo "[+] Successfully updated $ASOUND_CONF to use Card $SELECTED_CARD_NUM."
echo "[+] Reloading ALSA state..."
alsactl restore 2>/dev/null || true

echo "[✓] Done! Audio output switched."

# Optional test tone and mixer check
echo ""
read -rp "Would you like to play a test tone and check levels in alsamixer? [y/N]: " test_choice
if [[ "$test_choice" =~ ^[Yy]$ ]]; then
    echo "[+] Playing a short 440Hz sine wave test tone..."
    speaker-test -t sine -f 440 -l 1 &
    test_pid=$!
    
    sleep 0.5
    alsamixer -c "$SELECTED_CARD_NUM"
    
    kill "$test_pid" 2>/dev/null || wait "$test_pid" 2>/dev/null || true
fi
