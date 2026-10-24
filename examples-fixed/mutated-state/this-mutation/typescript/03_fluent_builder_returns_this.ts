class QueryBuilder {
  private filters: string[] = [];
  where(filter: string) {
    const next = new QueryBuilder();
    next.filters = [...this.filters, filter];
    return next;
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
