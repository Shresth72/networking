const { exec } = require("child_process");
const path = require("path");
const fs = require("fs");
const { S3Client, PutObjectCommand } = require("@aws-sdk/client-s3");
const mime = require("mime-types");
const { Kafka } = require("kafkajs");

const PROJECT_ID = process.env.PROJECT_ID;
const DEPLOYMENT_ID = process.env.DEPLOYMENT_ID;

// Kafka
const kafka = new Kafka({
  clientId: `docker-build-server-${DEPLOYMENT_ID}`,
  brokers: [process.env.KAFKA_BROKER_URL],
  ssl: {
    ca: [fs.readFileSync(path.join(__dirname, "kafka.pem"), "utf-8")],
  },
  sasl: {
    username: process.env.KAFKA_USERNAME,
    password: process.env.KAFKA_PASSWORD,
    mechanism: "plain",
  },
});

const producer = kafka.producer();

async function publishLog(log) {
  // publisher.publish(`logs:${PROJECT_ID}`, JSON.stringify({ log }));

  await producer.send({
    topic: `container-logs`,
    messages: [
      {
        key: "log",
        value: JSON.stringify({
          PROJECT_ID,
          DEPLOYMENT_ID,
          log,
        }),
      },
    ],
  });
}

// AWS S3
const s3Client = new S3Client({
  region: "ap-south-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});

async function init() {
  await producer.connect();

  console.log("Executing script.js");
  await publishLog("Build Started...");

  const outDirPath = path.join(__dirname, "output");

  // validate package.json to check if it's a valid node project

  const p = exec(`cd ${outDirPath} && npm install && ${"npm run build"}`);
  // build creates a dist folder with the build files to be deployed on S3 bucket

  // capture logs
  p.stdout.on("data", async (data) => {
    console.log(data.toString());
    await publishLog(data.toString());
  });

  p.stdout.on("error", async (error) => {
    console.log(`error: ${error.toString()}`);
    await publishLog(`error: ${error.toString()}`);
  });

  p.on("close", async () => {
    console.log("Script execution completed");
    await publishLog("Build Completed...");

    const distFolderPath = path.join(outDirPath, "dist");
    const distFolderContents = fs.readdirSync(distFolderPath, {
      recursive: true,
    });

    await publishLog("Uploading to S3...");
    for (const file of distFolderContents) {
      const filePath = path.join(distFolderPath, file);
      if (fs.lstatSync(filePath).isDirectory()) continue;

      await publishLog(`Uploading ${file}...`);

      const command = new PutObjectCommand({
        Bucket: "my-bucket",
        Key: `__outputs/${PROJECT_ID}/${file}`,
        Body: fs.createReadStream(filePath),
        ContentType: mime.lookup(filePath),
      });

      await s3Client.send(command);
      await publishLog(`Uploaded ${file}`);
    }

    await publishLog("Uploading Completed");
    console.log("Uploading completed");

    process.exit(0);
  });
}

init();