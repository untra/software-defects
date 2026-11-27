async function sendWebhook(event) {
    await Promise.resolve();
    throw new Error(`webhook endpoint returned 500 for ${event}`);
}
function completeOrder(id) {
    sendWebhook(`order.completed:${id}`).catch(error => console.log("webhook failed:", error.message));
    console.log("order", id, "completed");
}
completeOrder("A-100");
setTimeout(() => console.log("next order processed"), 10);
