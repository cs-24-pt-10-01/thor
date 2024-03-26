
// File system module
const fs = require('fs');
// Include Nodejs' net module.
const Net = require('net');
// The port number and hostname of the server.
const port = 6969;
const host = "127.0.0.1";

const client = new Net.Socket();

let dataBuffer = Buffer.alloc(0);


client.connect({ port: port, host: host }, function () {
    client.write("1"); // indicating client stream
    console.log('Connected');
});

client.on('data', function (data) {
    dataBuffer = Buffer.concat([dataBuffer, data]);

    try {
        //const temp = { data: dataString };
        const json = JSON.parse(dataBuffer.toString());
        console.log(json);

        // Writting to file
        writeJsonToFile(dataBuffer.toString(), 'data.json');

        // clearing buffer
        dataBuffer = Buffer.alloc(0);
    }
    catch (e) {
        console.log("Uncomplete Json, waiting for more data...");
    }
});

function writeJsonToFile(data, dist = 'data.json') {
    // if file exist append to file
    if (fs.existsSync(dist)) {
        // TODO fix json formatting, [],[] not allowed
        // Appending , to separate data
        fs.appendFile(dist, "," + data, 'utf8', function (err) {
            if (err) throw err;
        });
    }
    else {
        // write new file if the file does not exist
        fs.writeFile(dist, data, 'utf8', function (err) {
            if (err) throw err;
        });
    }

}
