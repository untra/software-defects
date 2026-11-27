class Connection {
    socket;
    connect() {
        this.socket = { write: (data) => console.log("sent", data) };
    }
    send(message) {
        if (!this.socket)
            throw new Error("connection is not ready");
        this.socket.write(message);
    }
}
const conn = new Connection();
conn.connect();
try {
    conn.send("hello");
}
catch (e) {
    console.log("error:", e.message);
}
