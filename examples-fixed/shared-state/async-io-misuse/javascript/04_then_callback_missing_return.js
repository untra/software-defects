function fetchUser(id) {
    return Promise.resolve({ name: ` user-${id} ` });
}
function loadUser(id) {
    return fetchUser(id).then((user) => {
        user.name = user.name.trim();
        return user;
    });
}
loadUser(7).then((user) => console.log("loaded:", user));
