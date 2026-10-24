class QueryBuilder {
  private filters: string[] = [];
  where(filter: string) {
    this.filters.push(filter);
    return this;
  }
  build() {
    return `SELECT * FROM users WHERE ${this.filters.join(" AND ")}`;
  }
}

const activeUsers = new QueryBuilder().where("active = true");
const admins = activeUsers.where("role = 'admin'");
const guests = activeUsers.where("role = 'guest'");
console.log(admins.build());
console.log(guests.build());
