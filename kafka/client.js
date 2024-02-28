const { Kafka } = require("kafkajs");

exports.kafka = new Kafka({
  clientId: "kafka_app",
  brokers: [`${process.env.IP_ADDRESS}:9092`]
});
