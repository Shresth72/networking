const express = require("express");
const { generateSlug } = require("random-word-slugs");
const { ECSClient, RunTaskCommand } = require("@aws-sdk/client-ecs");
const { Server } = require("socket.io");
const cors = require("cors");
const { z } = require("zod");
const { PrismaClient } = require("@prisma/client");
const { createClient } = require("@clickhouse/client");
const { Kafka } = require("kafkajs");
const { v4: uuidv4 } = require("uuid");
const fs = require("fs");
const path = require("path");

const app = express();
const PORT = 9000;

const prisma = new PrismaClient({});

const io = new Server({ cors: "*" });

// Kafka
const kafka = new Kafka({
  clientId: `api-server-${PORT}`,
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
const consumer = kafka.consumer({ groupId: "api-server-logs-consumer" });

// Clickhouse
const clickhouseClient = new createClient({
  host: process.env.CLICKHOUSE_HOST,
  database: "default",
  username: process.env.CLICKHOUSE_USER,
  password: process.env.CLICKHOUSE_PASSWORD,
});

io.on("connection", (socket) => {
  socket.on("subscribe", (channel) => {
    socket.join(channel);
    socket.emit("message", `Joined ${channel} channel`);
  });
});

io.listen(9001, () => console.log("Socket Server Running.. 9001"));

const ecsClient = new ECSClient({
  region: "ap-south-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});

const config = {
  CLUSTER: "cluster-arn",
  TASK: "task-arn",
};

app.use(express.json());
app.use(cors());

app.get("/logs/:id", async (req, res) => {
  const { id } = req.params;
  const logs = await clickhouseClient.query({
    query: `SELECT * FROM log_events WHERE deployment_id = {deployment_id:String}`,
    params: { deployment_id: id },
    format: "JSONEachRow",
  });

  const rawLogs = await logs.json();

  return res.json({ status: "success", data: rawLogs });
});

app.post("/project", async (req, res) => {
  const schema = z.object({
    name: z.string(),
    gitURL: z.string(),
  });
  const safeParseResult = schema.safeParse(req.body);

  if (safeParseResult.error)
    return res.status(400).json({ error: safeParseResult.error });

  const { name, gitURL } = safeParseResult.data;

  const deployment = await prisma.deployment.create({
    data: {
      name,
      gitURL,
      subdomain: generateSlug(),
    },
  });

  return res.json({ status: "success", data: deployment });
});

app.post("/deploy", async (req, res) => {
  const { projectId } = req.body;

  const project = await prisma.deployment.findUnique({
    where: { id: projectId },
  });
  if (!project) return res.status(404).json({ error: "Project not found" });

  // Check if there is no running deployment
  const runningDeployment = await prisma.deployment.findFirst({
    where: { status: { in: ["IN_PROGRESS", "QUEUED"] } },
  });
  if (runningDeployment)
    return res.status(400).json({ error: "Another deployment is in progress" });

  const deployment = await prisma.deployment.create({
    data: {
      project: { connect: { id: projectId } },
      status: "QUEUED",
    },
  });

  // Spin the container on ecs client
  // Copy the ARN of the cluster
  const command = new RunTaskCommand({
    cluster: config.CLUSTER,
    taskDefinition: config.TASK,
    launchType: "FARGATE",
    count: 1,
    networkConfiguration: {
      awsvpcConfiguration: {
        assignPublicIp: "ENABLED",
        subnets: ["subnet-0__", "subnet-1__", "subnet-2__"],
        securityGroups: ["sg-0__"],
      },
    },
    overrides: {
      containerOverrides: [
        {
          name: "builder-image",
          environment: [
            { name: "GIT_REPOSITORY_URL", value: project.gitURL },
            { name: "PROJECT_ID", value: projectId },
            { name: "DEPLOYMENT_ID", value: deployment.id.toString() },
          ],
        },
      ],
    },
  });

  await ecsClient.send(command);

  return res.json({
    status: "queued",
    data: { deploymentId: deployment.id },
  });
});

// async function initRedisSubscribe() {
//   console.log("Subscribing to logs:");
//   subscriber.psubscribe("logs:*");
//   subscriber.on("pmessage", (pattern, channel, message) => {
//     io.to(channel).emit("message", message);
//   });
// }

// initRedisSubscribe();

async function initKafkaConsumer() {
  await consumer.connect();
  await consumer.subscribe({ topics: ["container-logs"] });

  await consumer.run({
    autoCommit: false,
    eachBatch: async ({
      batch,
      heartbeat,
      resolveOffset,
      commitOffsetsIfNecessary,
    }) => {
      const messages = batch.messages;
      console.log(`Recv. ${messages.length} messages`);

      for (const message of messages) {
        const stringMessage = message.value.toString();
        const { PROJECT_ID, DEPLOYMENT_ID, log } = JSON.parse(stringMessage);
        console.log(`Received log: ${log} on deployment: ${DEPLOYMENT_ID}`);

        try {
          const { query_id } = await clickhouseClient.insert({
            tables: "log_events",
            values: [
              {
                event_id: uuidv4(),
                deployment_id: DEPLOYMENT_ID,
                log,
              },
            ],
            format: "JSONEachRow",
          });
          console.log(`Inserted into clickhouse: ${query_id}`);
          resolveOffset(message.offset);
          await commitOffsetsIfNecessary(message.offset);
          await heartbeat();
        } catch (error) {
          console.error(`Error: ${error}`);
        }
      }
    },
  });
}

initKafkaConsumer();

app.listen(PORT, () => console.log(`API Server Running.. ${PORT}`));
