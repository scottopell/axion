// Bootstrap script to load and initialize the WASM module

async function init() {
    try {
        console.log("Loading Xonix WASM module...");

        // Import the WASM module
        // The path will be './pkg/xonix.js' after building with wasm-pack
        const wasm = await import('./pkg/xonix.js');

        console.log("WASM module loaded successfully!");
        console.log("Initializing game...");

        // Initialize the WASM module first (loads the .wasm file)
        await wasm.default();

        console.log("WASM initialized, starting game...");

        // Now call our explicit start_game function
        wasm.start_game();

        console.log("Game started! Use arrow keys to play.");

    } catch (error) {
        console.error("Failed to load WASM module:", error);

        // Display error to user
        const container = document.querySelector('.game-container');
        if (container) {
            container.innerHTML = `
                <div class="loading" style="color: #ff5555;">
                    <p>Failed to load game</p>
                    <p style="font-size: 0.6em; margin-top: 10px;">
                        Error: ${error.message}
                    </p>
                    <p style="font-size: 0.5em; margin-top: 10px; color: #aaa;">
                        Make sure you've built the WASM module with: wasm-pack build --target web
                    </p>
                </div>
            `;
        }
    }
}

// Show loading message while WASM loads
const canvas = document.getElementById('gameCanvas');
if (canvas) {
    const ctx = canvas.getContext('2d');
    canvas.width = 640;
    canvas.height = 400;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = '#55ff55';
    ctx.font = '16px "Press Start 2P", monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('LOADING...', canvas.width / 2, canvas.height / 2);
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
