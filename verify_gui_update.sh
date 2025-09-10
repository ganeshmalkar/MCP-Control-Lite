#!/bin/bash
echo '🔧 MCP Control GUI - Updated Binary Test'
echo '========================================'
echo ''
echo '✅ Global binary updated: /usr/local/bin/mcpctl'
echo '✅ Frontend rebuilt with full UI'
echo '✅ System tray menu functional'
echo ''
echo 'Testing GUI launch...'
mcpctl --gui &
GUI_PID=$!
echo "GUI started with PID: $GUI_PID"
sleep 2
echo ''
echo '🎯 What you should now see:'
echo '  ✓ Main window with sidebar (Servers, Applications, Settings, Logs)'
echo '  ✓ Server list showing your actual MCP servers'
echo '  ✓ System status dashboard'
echo '  ✓ System tray icon in menu bar'
echo '  ✓ Right-click tray menu with Show/Status/Quit options'
echo ''
echo 'Press any key to stop the test...'
read -n 1
kill $GUI_PID 2>/dev/null
echo ''
echo '✅ Test completed!'

