import React from "react";
import ReactDOM from "react-dom/client";
import WidgetApp from "./WidgetApp.jsx";
import "../index.css";

const root = document.getElementById("root");
root.innerHTML = "";

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <WidgetApp />
  </React.StrictMode>
);