class Connection {
    socket;
    connect() {
        this.socket = { write: (data) => console.log("sent", data) };
    }
    send(message) {
        this.socket.write(message);
    }
}
const conn = new Connection();
try {
    conn.send("hello");
}
catch (e) {
    console.log("error:", e.message);
}
