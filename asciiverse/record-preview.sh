#!/bin/sh
# Records the catalog preview, asciiverse/glider-pro.cast: 100x30, Glider PRO's
# own demo (Demo House played from Calhoun's recorded keys, --demo), in the
# silhouette view, switching to the original view about a third of the way
# in and back at two thirds.
# Runs asciinema in a detached tmux pane, so nothing from this terminal gets
# in; a throwaway HOME and data folder keep the settings untouched.
#
#   cargo build --release && asciiverse/record-preview.sh
set -eu
cd "$(dirname "$0")/.."
out=asciiverse/glider-pro.cast
tmp=$(mktemp -d)
trap 'tmux kill-session -t gp-rec 2>/dev/null || true; rm -rf "$tmp"' EXIT
rm -f "$out"
tmux new-session -d -s gp-rec -x 100 -y 30 \
  "env -i PATH=$PWD/target/release:/usr/bin:/bin:/snap/bin HOME=$tmp USER=player LOGNAME=player \
     GLIDER_PRO_DATA_DIR=$tmp TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
   asciinema rec --window-size 100x30 --idle-time-limit 2 \
     --capture-env TERM,COLORTERM,LANG --title 'Glider PRO' \
     --command 'glider-pro --demo --mute --silhouette' $out"
sleep 22
tmux send-keys -t gp-rec v
sleep 16
tmux send-keys -t gp-rec v
sleep 16
tmux send-keys -t gp-rec C-c
sleep 2
ls -la "$out"
