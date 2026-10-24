type Socket = { write(data: string): void };

class Connection {
  socket?: Socket;
  connect() {
    this.socket = { write: (data: string) => console.log("sent", data) };
  }
  send(message: string) {
    if (!this.socket) throw new Error("connection is not ready");
    this.socket.write(message);
  }
}

const conn = new Connection();
conn.connect();
try {
  conn.send("hello");
} catch (e) {
  console.log("error:", (e as Error).message);
}
