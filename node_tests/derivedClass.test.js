const mod = require('./index.node');

it("runs", () => {
    expect(mod.test()).toBe(4);
})