const { kafka } = require("./client");
const group = process.argv[2];

async function init() {
  const consumer = kafka.consumer({ groupId: "rider-group" });
  await consumer.connect();

  await consumer.subscribe({ topic: "rider-updates" });

  await consumer.run({
    eachMessage: async ({ topic, partition, message, heartbeat, pause }) => {
      console.log(
        `${group}: [${topic}]: Partition:${partition}: ${message.value.toString()}`
      );
    }
  });

  await consumer.disconnect();
}

init();
