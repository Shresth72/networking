const express = require("express");
const httpProxy = require("http-proxy");

const app = express();
const PORT = 8000;

const BASE_PATH = `https://${process.env.BUCKET_NAME}.s3.ap-south-1.amazonaws.com/__outputs`;

const proxy = httpProxy.createProxy();

app.use((req, res) => {
  const hostname = req.hostname;
  const subdomain = hostname.split(".")[0];

  // Find the subdomain from DB and resolve it to the S3 bucket
  const id = prisma.deployment.findUnique({
    where: { subdomain },
  });

  // Add the subdomain to kafka topic to gain analytics

  const resolvesTo = `${BASE_PATH}/${id}`;

  proxy.web(req, res, { target: resolvesTo, changeOrigin: true });
});

proxy.on("proxyReq", (proxyReq, req, res) => {
  const url = req.url;
  if (url === "/") proxyReq.path += "index.html";
});

app.listen(PORT, () => console.log(`Reverse Proxy Running.. ${PORT}`));
