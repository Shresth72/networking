# Repository Deploy Tool

## Uploading Repositories on AWS

### Deploying the image of this deploy container tool to AWS ECR and run the task on AWS ECS

- Retrieve an authentication token and authenticate your Docker client to your registry. Use the AWS CLI:

```bash
aws ecr get-login-password --region ap-south-1 | docker login --username AWS --password-stdin <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com
```

- Build your Docker image using the following command. For information on building a Docker file from scratch see the instructions here: [docker/engine/builder](https://docs.docker.com/engine/reference/builder/)

```bash
docker build -t <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name> .
```

- After the build completes, tag your image so you can push the image to this repository:

```bash
docker tag <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name>:latest
```

- Run the following command to push this image to your newly created AWS repository:

```bash
docker push <aws_account_id>.dkr.ecr.ap-south-1.amazonaws.com/<project_name>:latest
```

### Uploading any repository files on EC2

- Create a cluster on AWS ECS to deploy the container
- Copy the URI of the container from the AWS ECR
- Create a new task definition to run the image in the container

### To test it locally

- Have AWS IAM keys setup
- To run it locally on docker instead of deploying it on ECR
  
```js
sudo docker build -t <docker_image_name> .

sudo docker run -it -e GIT_REPOSITORY_URL=<git_repo_url> -e PROJECT_ID=<project_id> <docker_image_name>
```

## Setting up S3 Reverse Proxy

- Setup a nginx or httpProxy to handle requests from the ECS URL
