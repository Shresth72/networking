const { exec } = require("child_process");
const path = require("path");
const fs = require("fs");
const { S3Client, PutObjectCommand } = require("@aws-sdk/client-s3");
const mime = require("mime-types");

const s3Client = new S3Client({
  region: "ap-south-1",
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  },
});

const PROJECT_ID = process.env.PROJECT_ID;

async function init() {
  console.log("Executing script.js");

  const outDirPath = path.join(__dirname, "output");

  const p = exec(`cd ${outDirPath} && npm install && npm run build`);
  // build creates a dist folder with the build files to be deployed on S3 bucket

  // capture logs
  p.stdout.on("data", (data) => {
    console.log(data.toString());
  });

  p.stdout.on("error", (error) => {
    console.log(`Error: ${error.toString()}`);
  });

  p.on("close", async () => {
    console.log("Script execution completed");
    const distFolderPath = path.join(outDirPath, "dist");
    const distFolderContents = fs.readdirSync(distFolderPath, {
      recursive: true,
    });

    for (const file of distFolderContents) {
      const filePath = path.join(distFolderPath, file);
      if (fs.lstatSync(filePath).isDirectory()) continue;

      const command = new PutObjectCommand({
        Bucket: "my-bucket",
        Key: `__outputs/${PROJECT_ID}/${filePath}`,
        Body: fs.createReadStream(filePath),
        ContentType: mime.lookup(filePath),
      });

      await s3Client.send(command);
    }

    console.log("Deployment completed");
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