// --- THIS LINE IS CRITICAL ---
// It tells the bundler (Vite) to include all the styles.
import './style.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
window.addEventListener("DOMContentLoaded", async () => {
    const dropZone = document.getElementById("drop-zone");
    const statusEl = document.getElementById("status");
    const ipInput = document.getElementById("ip-input");
    const fileNameDisplay = document.getElementById("file-name-display");
    const sendBtn = document.getElementById("send-btn");
    let selectedFilePath = null;
    // --- UI Update Function ---
    const setFileReady = (filePath) => {
        selectedFilePath = filePath;
        // Extract just the file name from the full path
        const fileName = filePath.substring(filePath.lastIndexOf('\\') + 1).substring(filePath.lastIndexOf('/') + 1);
        fileNameDisplay.textContent = fileName;
        dropZone.classList.add('file-selected');
    };
    const resetUi = () => {
        selectedFilePath = null;
        dropZone.classList.remove('file-selected');
        dropZone.classList.remove('is-processing');
    };
    // --- Event Listeners ---
    // 1. Click the portal to select a file
    dropZone.addEventListener('click', async () => {
        // Don't open dialog if a file is already selected or processing
        if (selectedFilePath || dropZone.classList.contains('is-processing'))
            return;
        try {
            const path = await invoke('open_file_dialog');
            setFileReady(path);
        }
        catch (err) {
            console.error("File selection cancelled or failed:", err);
            resetUi();
        }
    });
    // 2. Drag and drop a file
    listen('tauri://file-drop', (event) => {
        dropZone.classList.remove('drag-over');
        if (event.payload.length > 0) {
            setFileReady(event.payload[0]);
        }
    });
    // 3. Click the "Send File" button
    sendBtn.addEventListener('click', async () => {
        const targetIp = ipInput.value.trim();
        if (!selectedFilePath) {
            statusEl.textContent = 'Error: No file selected.';
            return;
        }
        if (!targetIp) {
            statusEl.textContent = 'Error: Please enter a target IP address.';
            return;
        }
        // --- UI for Processing ---
        statusEl.textContent = `Engaging portal for ${targetIp}...`;
        dropZone.classList.add('is-processing');
        dropZone.classList.remove('file-selected');
        sendBtn.disabled = true;
        try {
            await invoke("send_file", { filePath: selectedFilePath, targetIp });
            statusEl.textContent = "Transfer successful!";
        }
        catch (error) {
            console.error("Backend returned an error:", error);
            statusEl.textContent = `Error: ${error}`;
        }
        finally {
            // --- Reset UI after completion ---
            resetUi();
            sendBtn.disabled = false;
            setTimeout(() => { statusEl.textContent = ''; }, 4000);
        }
    });
    // Drag-over visual feedback listeners
    listen('tauri://file-drop-hover', () => dropZone.classList.add('drag-over'));
    listen('tauri://file-drop-cancelled', () => dropZone.classList.remove('drag-over'));
});
//# sourceMappingURL=main.js.map