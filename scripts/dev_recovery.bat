@echo off
echo 🚀 Tadpole OS - Dev Mode Agent Recovery
echo ---------------------------------------
echo 1. Ensuring workflows are in place...
if not exist "data\workflows" mkdir "data\workflows"
echo 2. Running Agent Import Script...
python scripts\import_agents_legacy.py
python scripts\quiesce_swarm.py
echo.
echo ---------------------------------------
echo 🎉 Recovery execution finished! 
echo Check the application UI to see your agents.
pause
