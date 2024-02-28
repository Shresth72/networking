const { kafka } = require("./client");
const dotenv = require("dotenv");
dotenv.config();

async function init() {
  const admin = kafka.admin();
  console.log("Admin Connecting...");
  admin.connect();
  console.log("Admin Connected Successfully...");

  // Create Kafka Topics
  await admin.createTopics({
    topics: [
      {
        topic: "rider-updates",
        numPartitions: 2
      }
    ]
  });

  await admin.disconnect();
}

init();
