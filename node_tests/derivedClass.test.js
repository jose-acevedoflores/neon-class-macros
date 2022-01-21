const mod = require("./index.node");

const p = "some_path";
const val = "Le_VAL";
const map = {
  map: [{ k: "LE_KEY", v: val }],
};

describe("TestStruct", () => {
  test("constructor", () => {
    const obj = new mod.TestStruct(p, map);
    expect(obj).toBeDefined();
  });

  describe("Calling exported methods", () => {
    const obj = new mod.TestStruct(p, map);

    it("calls 'start_camel'", async () => {
      const res = await obj.startCamel(4);
      expect(res).toBe(8);
    });

    it("calls 'another_one'", () => {
      const result = obj.anotherOne(21, "from-js");
      expect(result).toEqual(`hehe from-js-21-"${p}"`);
    });

    it("calls 'plain_method'", () => {
      const arg = 37.12;
      const res = obj.plainMethod(arg);
      expect(res).toBe(`to-str-${arg}-${val}`);
    });

    it("calls 'method_that_returns_nothing'", () => {
      expect(obj.methodThatReturnsNothing()).toBeUndefined();
    });

    it("calls 'take_numeric' result ok", () => {
      expect(obj.takeNumericReturnResult(123, -3123)).toBe(-3000);
    });

    it("calls 'take_numeric' and throws", () => {
      expect(() => obj.takeNumericReturnResult(0, -1)).toThrow(
        "Second arg was -1"
      );
    });

    it("calls 'take_cx_but_return_native_val'", () => {
      const arg = 3.2;
      const res = obj.takeCxButReturnNativeVal(arg);
      expect(res).toBe(`to-str-${arg}-${val}`);
    });
  });

  test("to_js_obj via the 'test' rust function", async () => {
    const path_num = 3;
    const p = `random_path_${path_num}`;
    const ts = await mod.test(3);
    const arg = 12.8;
    const res = ts.plainMethod(arg);
    expect(res).toBe(`to-str-${arg}-NONE`);

    const result = ts.anotherOne(2122, "from-js");
    expect(result).toEqual(`hehe from-js-2122-"${p}"`);
  });
});

describe("call 'standalone_function' with throw_on_err", () => {
  test("result ok", () => {
    const arg = "js";
    const res = mod.standaloneFunction(arg, 23.12);
    expect(res).toBe(`Got some stuff from ${arg}`);
  });

  test("result err", () => {
    expect(() => mod.standaloneFunction("", 100)).toThrow("bad num");
  });
});

describe("TestStruct2", () => {
  test("constructor_with_cx", (done) => {
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

  test("call 'method_with_neon_args'", (done) => {
    const obj = new mod.TestStruct2(p, map, () => {});
    const cbArg = 3;
    const echo = "raw_js_value";
    const cb = (arg, echoed) => {
      try {
        expect(arg).toBe(`mixed-"${p}"-${cbArg}`);
        expect(echoed).toBe(echo);
        done();
      } catch (error) {
        done(error);
      }
    };
    obj.methodWithNeonArgs(cb, cbArg, echo);
  });
});
