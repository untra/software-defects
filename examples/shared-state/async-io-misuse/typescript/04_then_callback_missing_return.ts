type User = { name: string };

function fetchUser(id: number): Promise<User> {
  return Promise.resolve({ name: ` user-${id} ` });
}

function loadUser(id: number): Promise<User | void> {
  return fetchUser(id).then((user) => {
    user.name = user.name.trim();
  });
}

loadUser(7).then((user) => console.log("loaded:", user));
