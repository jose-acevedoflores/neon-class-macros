// NOTE: Since `cargo publish` won't include files that are part of the `node_tests` sub crate, we need to move the relevant
// files to the `docs` folder. This helps with keeping the docs 'live' since node_tests are executed with `cargo t`

const path = require("path");
const fs = require("fs-extra");

const config = require("../package.json");

const distDir = path.join(__dirname, "..", "docs");

config.live_docs.forEach((item_path) => {
  const out = path.join(distDir, item_path);
  fs.copySync(item_path, out);
});
