"use strict";
/* eslint-disable */
if (process.env.NODE_ENV == "development") {
    const script = document.createElement("script");
    script.src = "http://localhost:8098";
    document.head.appendChild(script);
}
