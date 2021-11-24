const mod = require("./index.node");

it("basic constructor", () => {
  const p = "some_path";
  const obj = new mod.TestStruct(p, {
    map: [{ k: "LE_KEY", v: "Le_VAL" }],
  });
  expect(obj).toBeDefined();
  const result = obj.anotherOne(21, "from-js");
  expect(result).toEqual(`hehe from-js-21-"${p}"`);
});

it("runs", () => {
  expect(mod.test()).toBe(4);
});
