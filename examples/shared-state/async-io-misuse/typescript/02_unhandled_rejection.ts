async function sendWebhook(event: string): Promise<void> {
  await Promise.resolve();
  throw new Error(`webhook endpoint returned 500 for ${event}`);
}

function completeOrder(id: string): void {
  sendWebhook(`order.completed:${id}`);
  console.log("order", id, "completed");
}

completeOrder("A-100");
setTimeout(() => console.log("next order processed"), 10);
