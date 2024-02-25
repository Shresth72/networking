const SMTPServer = require("smtp-server").SMTPServer;

// To handle handshake, smtp server sends callbacks
const server = new SMTPServer({
  allowInsecureAuth: true,
  authOptional: true,
  onConnect(session, callback) {
    console.log(`onConnect`, session.id);
  },
  onMailFrom(address, session, callback) {
    console.log(`onMailFrom`, address.address, session.id);
    callback();
  },
  onRcptTo(address, session, callback) {
    console.log(`onRcptTo`, address.address, session.id);
    // Check in db if the recipient exists
    callback();
  },
  onData(stream, session, callback) {
    console.log(`onData`, session.id);

    // stream.pipe(process.stdout);
    stream.on("data", (data) => console.log(data.toString()));
    stream.on("end", callback);
  }
});

server.listen(25, () => console.log("SMTP server started on port 25"));

/* Host on AWS EC2
    -> Launch an Ubuntu instance and create key pair for connecting to the instance
    -> Set Security Group to allow traffic on port 25 with SMTP protocol
    -> Install Node.js and SMTP server
    -> Create a working directory and copy the code

    -> Setup Mail Server on CloudFlare for MX records
    -> Setup DNS records for the domain with A record pointing to the EC2 instance
    -> Setup SPF and DKIM signature for the domain
    -> Setup DMARC record for the domain to receive reports on email delivery
*/