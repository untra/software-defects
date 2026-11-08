type Socket = { write(data: string): void };

class Connection {
  socket!: Socket;
  connect() {
    this.socket = { write: (data: string) => console.log("sent", data) };
  }
  send(message: string) {
    this.socket.write(message);
  }
}

const conn = new Connection();
try {
  conn.send("hello");
} catch (e) {
  console.log("error:", (e as Error).message);
}
