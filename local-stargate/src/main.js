import './style.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
window.addEventListener("DOMContentLoaded", () => {
    const dropZone = document.getElementById("drop-zone");
    const statusEl = document.getElementById("status");
    const dropZoneText = document.getElementById("drop-zone-text");
    // Get the new IP input element
    const ipInput = document.getElementById("ip-input");
    const showDragIndicator = () => dropZone.classList.add('drag-over');
    const hideDragIndicator = () => dropZone.classList.remove('drag-over');
    listen('tauri://file-drop-hover', showDragIndicator);
    listen('tauri://file-drop-cancelled', hideDragIndicator);
    listen('tauri://file-drop', (event) => {
        const filePaths = event.payload;
        // Get the IP address from the input field
        const targetIp = ipInput.value.trim();
        console.log(`Files dropped: ${filePaths}, Target IP: ${targetIp}`);
        hideDragIndicator();
        // Validate that an IP address has been entered
        if (!targetIp) {
            statusEl.textContent = 'Error: Please enter a target IP address.';
            setTimeout(() => { statusEl.textContent = ''; }, 4000);
            return;
        }
        if (filePaths.length > 0) {
            statusEl.textContent = `Engaging portal for ${targetIp}...`;
            dropZone.classList.add('is-processing');
            dropZoneText.textContent = 'Transferring...';
            // Call the backend, now with the targetIp
            invoke("file_dropped", { filePath: filePaths[0], targetIp: targetIp })
                .then(() => {
                console.log("Backend acknowledged the file.");
                statusEl.textContent = "Transfer successful!";
            })
                .catch((error) => {
                console.error("Backend returned an error:", error);
                statusEl.textContent = `Error: ${error}`;
            })
                .finally(() => {
                dropZone.classList.remove('is-processing');
                dropZoneText.textContent = 'Drag & Drop Files';
                setTimeout(() => { statusEl.textContent = ''; }, 4000);
            });
        }
    });
});
//# sourceMappingURL=main.js.map