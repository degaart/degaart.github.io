"use strict";

function onClick() {
    const resultDiv = document.getElementById("result-div");

    let img;
    let text;
    switch(Math.floor(Math.random() * 3)) {
        case 0:
            img = "img/beer.jpg";
            text = "Oui, bière";
            break;
        case 1:
            img = "img/no-beer.jpg";
            text = "Non, pas bière";
            break;
        case 2:
            img = "img/beer-pussy.jpg";
            text = "Bière avec chattes";
            break;
    }

    resultDiv.innerHTML = "<p><strong>" + text + "</strong></p>" + "<p><img src=\"" + img + "\"></p>";
}

function onLoad() {
    document.getElementById("biere-btn").addEventListener("click", onClick);
}

window.addEventListener("load", onLoad);
