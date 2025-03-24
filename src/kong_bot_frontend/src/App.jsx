import React, { useState } from "react";

function App() {
    const [botStarted, setBotStarted] = useState(false);
   
    return (
        <div>
            <h1>Welcome to Kong Bot</h1>
            {!botStarted ? (
                <button onClick={() => setBotStarted(true)}>Start</button>
            ) : (
                <p>Bot Started! Use /help for commands.</p>
            )}
        </div>
    );
}

export default App;