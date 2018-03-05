const exonum = require('exonum-client');
const request = require('request');

let sendFunds = exonum.newMessage({
    network_id: 0,
    protocol_version: 0,
    service_id: 1,
    message_id: 0,
    size: 40,
    fields: {
        pub_key: {type: exonum.PublicKey, size: 32, from: 0, to: 32},
        name: {type: exonum.String, size: 8, from: 32, to: 40}
    }
});

function sendFile(fileName) {
    let keyPair = exonum.keyPair();

    let data = {
        pub_key: keyPair.publicKey,
        name: fileName
    };

    let requestBody = {
        body: data,
        network_id: 0,
        protocol_version: 0,
        service_id: 1,
        message_id: 0,
        signature: sendFunds.sign(keyPair.secretKey, data)
    };

    console.log('Sending transaction ' + fileName + '...');
    request.post({
        headers: {
            'Content-Type': 'application/json'
        },
        url: 'http://127.0.0.1:8000/api/services/timestamping/v1/timestamps',
        body: JSON.stringify(requestBody)
    }, (err, res, body) => {
        if (err) {
            return console.log(err);
        }
        console.log(body.url);
        console.log(body.explanation);
    });
}

function runTest() {
    // 1k per second
    setInterval(function () {
        for (let i = 0; i < 10; ++i) {
            sendFile('rnd_' + exonum.randomUint64() + '.txt');
        }
    }, 10);
}

function addSome() {
    for (let i = 0; i < 100; ++i) {
        sendFile('rnd_' + exonum.randomUint64() + '.txt');
    }
}

