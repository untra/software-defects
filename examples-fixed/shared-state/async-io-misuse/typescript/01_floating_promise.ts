const WRITE_LATENCY_MS = 10;

const profiles = new Map<string, string>([["ana", "old bio"]]);

async function persistBio(user: string, bio: string): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, WRITE_LATENCY_MS));
  profiles.set(user, bio);
}

async function updateBio(user: string, bio: string): Promise<string> {
  await persistBio(user, bio);
  return "saved";
}

updateBio("ana", "new bio").then(result => {
  console.log(result);
  console.log("profile page shows:", profiles.get("ana"));
});
