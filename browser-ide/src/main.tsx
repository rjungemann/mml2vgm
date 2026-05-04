import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Note: Monaco Editor CSS is loaded from CDN in index.html
// The @monaco-editor/react package handles Monaco initialization

// Initialize the app
ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);
