const mod = require("./index.node");

it("basic constructor", () => {
  const obj = new mod.TestStruct("some_path", {
    map: [{ k: "LE_KEY", v: "Le_VAL" }],
  });
  expect(obj).toBeDefined();
});

it("runs", () => {
  expect(mod.test()).toBe(4);
});
