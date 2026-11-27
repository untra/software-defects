const MAX_DEPTH = 3;
type Group = { name: string; members: Group[] };

function count(group: Group, depth = 1): number {
  if (depth > MAX_DEPTH) throw new Error("group depth exceeded");
  let total = 1;
  for (const member of group.members) {
    total += count(member, depth + 1);
  }
  return total;
}

const root: Group = {
  name: "root",
  members: [
    {
      name: "east",
      members: [{ name: "desk", members: [{ name: "bin", members: [] }] }],
    },
  ],
};

try { console.log("groups", count(root)); } catch (error) { console.log((error as Error).message); }
