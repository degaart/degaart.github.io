"use strict";

let sack = Array();
let lastChoice = -1;

function onClick() {
    const resultDiv = document.getElementById("result-div");

    let choice;
    do {
        choice = Math.floor(Math.random() * 3);
    } while(choice == lastChoice);
    lastChoice = choice;

    let img;
    let text;
    switch(choice) {
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
