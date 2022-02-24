const OUT1 = document.getElementById('1-out');
const OUT2 = document.getElementById('2-out');
const OUT3 = document.getElementById('3-out');

const endpoint = 'https://lab.trendmicroservices.sohamroy.me'

const ADDR1 = endpoint + '/adder/adder/add';
const ADDR2 = endpoint + '/rand/rand/rand';
const ADDR3 = endpoint + '/multiplier/multiplier/multiply';

const service1 = async () => {
    let x = Math.round(Math.random() * 200 - 100);
    let y = Math.round(Math.random() * 200 - 100);

    let req = {
        x: x,
        y: y,
    }

    let res = await fetch(ADDR1, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(req),
    })
        .then(res => res.json());

    OUT1.innerText = `${x} + ${y} = ${res.result}`;
};

const service2 = async () => {
    let res = await fetch(ADDR2)
        .then(res => res.json());

    OUT2.innerText = `${res.result}`;
};

const service3 = async () => {
    let x = Math.round(Math.random() * 200 - 100);
    let y = Math.round(Math.random() * 200 - 100);

    let req = {
        x: x,
        y: y,
    }

    let res = await fetch(ADDR3, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify(req),
    })
        .then(res => res.json());

    OUT3.innerText = `${x} * ${y} = ${res.result}`;
};