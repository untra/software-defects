type Group = { name: string; members: Group[] };

function count(group: Group): number {
  let total = 1;
  for (const member of group.members) {
    total += count(member);
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

console.log("groups", count(root));
