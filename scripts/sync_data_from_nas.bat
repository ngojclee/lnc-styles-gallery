@echo off
setlocal
REM Sync read-only corpora from NAS into local ./data directory for lnc-styles-gallery

set "ROOT_DIR=%~dp0.."
set "DATA_DIR=%ROOT_DIR%\data"
set "NAS_REFERO=\\10.21.2.2\data\_agentsync\repos\refero-styles"
set "NAS_PROMPT_UI=\\10.21.2.2\data\_agentsync\repos\prompt-driven-ui"

if not exist "%DATA_DIR%" mkdir "%DATA_DIR%"

echo [1/2] Syncing refero-styles from %NAS_REFERO% ...
robocopy "%NAS_REFERO%" "%DATA_DIR%\refero-styles" /MIR /R:2 /W:2 /MT:8 /XD .git /NFL /NDL

echo [2/2] Syncing prompt-driven-ui from %NAS_PROMPT_UI% ...
robocopy "%NAS_PROMPT_UI%" "%DATA_DIR%\prompt-driven-ui" /MIR /R:2 /W:2 /MT:8 /XD .git node_modules /NFL /NDL

echo Done. Trigger POST /api/admin/reindex to refresh in-memory index without restart.
exit /b 0
