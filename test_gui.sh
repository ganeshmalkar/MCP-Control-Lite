#!/bin/bash
echo 'Testing MCP Control GUI...'
echo 'Starting GUI in background...'
./src-tauri/target/release/mcpctl --gui &
GUI_PID=$!
echo "GUI started with PID: $GUI_PID"
sleep 3
echo 'Checking if GUI process is running...'
if ps -p $GUI_PID > /dev/null; then
    echo '✅ GUI is running successfully!'
    echo 'You should see:'
    echo '  1. A window titled "MCP Control" with server information'
    echo '  2. A system tray icon (small icon in menu bar)'
    echo '  3. Right-click the tray icon to see menu options'
    echo ''
    echo 'Press any key to stop the GUI...'
    read -n 1
    kill $GUI_PID
    echo 'GUI stopped.'
else
    echo '❌ GUI failed to start or crashed'
fi
