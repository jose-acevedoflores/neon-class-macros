const mod = require("./index.node");

const p = "some_path";
const val = "Le_VAL";
const map = {
  map: [{ k: "LE_KEY", v: val }],
};

describe("TestStruct", () => {
  it("basic constructor", () => {
    const obj = new mod.TestStruct(p, map);
    expect(obj).toBeDefined();
  });

  describe("Calling generated methods", () => {
    const obj = new mod.TestStruct(p, map);

    it("calls method that had cx as second arg", () => {
      const result = obj.anotherOne(21, "from-js");
      expect(result).toEqual(`hehe from-js-21-"${p}"`);
    });

    it("calls method that didn't have cx as second arg", () => {
      const arg = 37.12;
      const res = obj.plainMethod(arg);
      expect(res).toBe(`to-str-${arg}-${val}`);
    });

    it("calls method that didn't have cx as second arg and returns nothing", () => {
      expect(obj.methodThatReturnsNothing()).toBeUndefined();
    });

    it("Check numeric values", () => {
      expect(obj.takeNumeric(123, -3123)).toBe(-3000);
    });
  });

  it("check to_js_obj", async () => {
    const ts = await mod.test();
    const arg = 12.8;
    const res = ts.plainMethod(arg);
    expect(res).toBe(`to-str-${arg}-NONE`);
  });
});

describe("TestStruct2", () => {
  it("basic constructor with lambda argument", (done) => {
    const obj = new mod.TestStruct2(p, map, (arg) => {
      try {
        expect(arg).toBe(`called from rust thread-"${p}"`);
        done();
      } catch (error) {
        done(error);
      }
    });
    expect(obj).toBeDefined();
  });
});
