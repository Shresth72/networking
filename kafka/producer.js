const { kafka } = require("./client");
const readline = require("readline");

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

async function init() {
  const producer = kafka.producer();

  console.log("Producer Connecting...");
  await producer.connect();
  console.log("Producer Connected Successfully...");

  rl.setPrompt(">");
  rl.prompt();

  rl.on("line", async (line) => {
    const [riderName, location] = line.split(" ");

    // Send Messages
    await producer.send({
      topic: "rider-updates",
      messages: [
        {
          partition: location.toLowerCase() === "north" ? 0 : 1,
          key: "location-update",
          value: JSON.stringify({
            rider_id: 1,
            name: riderName,
            loc: location
          })
        }
      ]
    });
  }).on("close", async () => {
    await producer.disconnect();
  });
}

init();
