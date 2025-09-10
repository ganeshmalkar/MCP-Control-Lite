#!/bin/bash
echo '🔧 Testing System Tray Menu Functionality...'
echo ''
echo 'Starting MCP Control GUI...'
./src-tauri/target/release/mcpctl --gui &
GUI_PID=$!
echo "✅ GUI started with PID: $GUI_PID"
echo ''
echo '📋 System Tray Menu Test Instructions:'
echo '1. Look for the MCP Control icon in your menu bar (top-right)'
echo '2. RIGHT-CLICK on the tray icon'
echo '3. You should see a menu with these options:'
echo '   • Show MCP Control'
echo '   • System Status'  
echo '   • Quit'
echo '4. LEFT-CLICK on the tray icon should show the main window'
echo ''
echo '🎯 Expected Menu Options:'
echo '   ✓ Show MCP Control - Opens/focuses the main window'
echo '   ✓ System Status - Opens the main window'
echo '   ✓ Quit - Closes the application'
echo ''
echo 'Press any key when you have tested the menu...'
read -n 1
echo ''
echo 'Stopping GUI...'
kill $GUI_PID 2>/dev/null
echo '✅ Test completed!'

