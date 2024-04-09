
// File system module
const fs = require('fs');
// Include Nodejs' net module.
const Net = require('net');
// The port number and hostname of the server.
const port = 6969;
const host = "127.0.0.1";
const endString = "end";
const end = new Buffer.from(endString);
const repo = "https://github.com/cs-24-pt-10-01/HotspotBenchmarkJS.git";

const client = new Net.Socket();

let dataBuffer = Buffer.alloc(0);

client.connect({ port: port, host: host }, function () {
    client.write("1"); // indicating client stream
    client.write(repo);
    console.log('Connected');
});

client.on('data', function (data) {
    if (data.subarray(data.length - end.length).toString() == endString) {
        dataBuffer = Buffer.concat([dataBuffer, data]);

        const dataBufferString = dataBuffer.toString().slice(0, -end.length);

        const json = JSON.parse(dataBufferString);

        // Writting to file
        writeJsonToFile(dataBufferString, 'data.json');

        // clearing buffer
        dataBuffer = Buffer.alloc(0);
    }
    else {
        dataBuffer = Buffer.concat([dataBuffer, data]);
    }
});

client.on('error', function (err) {
    console.log(err);
});

client.on('end', function () {
    console.log('Disconnected');
});

// TODO fix json formatting, "[...],[...]" not allowed
function writeJsonToFile(data, dist = 'data.json') {
    console.log("Writing to file...");
    // if file exist append to file
    if (fs.existsSync(dist)) {
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
