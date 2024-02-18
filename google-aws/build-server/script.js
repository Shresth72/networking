const { exec } = require("child_process");
const path = require("path");
const fs = require("fs");
const { S3Client, PutObjectCommand } = require("@aws-sdk/client-s3");
const mime = require("mime-types");
const Redis = require("ioredis");

const PROJECT_ID = process.env.PROJECT_ID;

// Redis
const publisher = new Redis(process.env.REDIS_URL);

function publishLog(log) {
  publisher.publish(`logs:${PROJECT_ID}`, JSON.stringify({ log }));
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
  console.log("Executing script.js");
  publishLog("Build Started...");

  const outDirPath = path.join(__dirname, "output");

  const p = exec(`cd ${outDirPath} && npm install && npm run build`);
  // build creates a dist folder with the build files to be deployed on S3 bucket

  // capture logs
  p.stdout.on("data", (data) => {
    console.log(data.toString());
    publishLog(data.toString());
  });

  p.stdout.on("error", (error) => {
    console.log(`error: ${error.toString()}`);
    publishLog(`error: ${error.toString()}`);
  });

  p.on("close", async () => {
    console.log("Script execution completed");
    publishLog("Build Completed...");

    const distFolderPath = path.join(outDirPath, "dist");
    const distFolderContents = fs.readdirSync(distFolderPath, {
      recursive: true,
    });

    publishLog("Uploading to S3...");
    for (const file of distFolderContents) {
      const filePath = path.join(distFolderPath, file);
      if (fs.lstatSync(filePath).isDirectory()) continue;

      publishLog(`Uploading ${file}...`);

      const command = new PutObjectCommand({
        Bucket: "my-bucket",
        Key: `__outputs/${PROJECT_ID}/${file}`,
        Body: fs.createReadStream(filePath),
        ContentType: mime.lookup(filePath),
      });

      await s3Client.send(command);
      publishLog(`Uploaded ${file}`);
    }

    publishLog("Uploading Completed");
    console.log("Uploading completed");
  });
}

/*
  Push the container on AWS Elastic Container Registry

  -> Retrieve an authentication token and authenticate your Docker client to your registry.
     Use the AWS CLI:
      aws ecr get-login-password --region ap-south-1 | docker login --username AWS --password-stdin <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com

  -> Build your Docker image using the following command. For information on building a Docker file from scratch see the instructions here: https://docs.docker.com/engine/reference/builder/
      docker build -t <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name> .

  -> After the build completes, tag your image so you can push the image to this repository:
      docker tag <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name>:latest

  -> Run the following command to push this image to your newly created AWS repository:
      docker push <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name>:latest
*/

/*
  Create a cluster on AWS ECS to deploy the container

  -> Copy the URI of the container from the AWS ECR
  -> Create a new task definition to run the image in the container

*/

// To Test locally run the following command
// sudo docker build -t <docker_image_name> .
// sudo docker run -it -e GIT_REPOSITORY_URL=<git_repo_url> -e PROJECT_ID=<project_id> <docker_image_name>
