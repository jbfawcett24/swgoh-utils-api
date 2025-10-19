document.querySelector(".characters").addEventListener("submit", getCharacters)
document.querySelector(".account").addEventListener("submit", getAccount)


async function getCharacters(e) {
    e.preventDefault();
    charId = document.querySelector("#charId").value;
    console.log(charId);

    let options = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json' 
        },
        body: JSON.stringify({
            charId: charId
        })
    }

    let response = await fetch("http://localhost:7474/characters", options);

    let data = await response.json();
    console.log(data);
    setOutput(data);
}

async function getAccount(e) {
    e.preventDefault();
    allyCode = document.querySelector("#allyCode").value;

    let options = {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json' 
        },
        body: JSON.stringify({
            allyCode: allyCode
        })
    }

    let response = await fetch("http://localhost:7474/account", options);

    let data = await response.json();

    setOutput(data);
}

function setOutput(data) {
    document.querySelector(".output").innerText = JSON.stringify(data, null, 2);
}