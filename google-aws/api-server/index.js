const express = require("express");
const { generateSlug } = require("random-word-slugs");
const { ECSClient, RunTaskCommand } = require("@aws-sdk/client-ecs");
const { Server } = require("socket.io");
const cors = require("cors");
const { z } = require("zod");
const { PrismaClient } = require("@prisma/client");
const { createClient } = require("@clickhouse/client");

const app = express();
const PORT = 9000;

const prisma = new PrismaClient({});

const io = new Server({ cors: "*" });

const client = new createClient({
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
    data: { projectSlug, url: `http://${projectSlug}.localhost:8000` },
  });
});

async function initRedisSubscribe() {
  console.log("Subscribing to logs:");
  subscriber.psubscribe("logs:*");
  subscriber.on("pmessage", (pattern, channel, message) => {
    io.to(channel).emit("message", message);
  });
}

initRedisSubscribe();

app.listen(PORT, () => console.log(`API Server Running.. ${PORT}`));
