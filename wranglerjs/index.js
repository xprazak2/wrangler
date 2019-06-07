const webpack = require("webpack");
const MemoryFS = require("memory-fs");
const { join } = require("path");
const { writeFileSync, readFileSync } = require("fs");
// const FetchCompileAsyncWasmPlugin = require("webpack/lib/web/FetchCompileAsyncWasmPlugin");
const AsyncWasmChunkLoadingRuntimeModule = require("webpack/lib/wasm-async/AsyncWasmChunkLoadingRuntimeModule");
const RuntimeGlobals = require("webpack/lib/RuntimeGlobals");

const rawArgs = process.argv.slice(2);
const args = rawArgs.reduce((obj, e) => {
  if (e.indexOf("--") === -1 && e.indexOf("=") === -1) {
    throw new Error("malformed arguments");
  }

  const [name, value] = e.split("=");
  const normalizedName = name.replace("--", "");
  obj[normalizedName] = value;
  return obj;
}, {});

let config;
if (args["no-webpack-config"] === "1") {
  config = { entry: args["use-entry"] };
} else {
  config = require(join(process.cwd(), "./webpack.config.js"));
}

const fs = new MemoryFS();
const compiler = webpack(config);
const fullConfig = compiler.options;

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

// Override the {FetchCompileAsyncWasmPlugin} and inject our new runtime.
const [
  fetchCompileAsyncWasmPlugin
] = compiler.hooks.thisCompilation.taps.filter(
  tap => tap.name === "FetchCompileAsyncWasmPlugin"
);
// fetchCompileAsyncWasmPlugin.fn = function(compilation) {
//   const generateLoadBinaryCode = () => `
//       // Fake fetch response
//       Promise.resolve({
//         arrayBuffer() { return Promise.resolve(${args["wasm-binding"]}); }
//       });
//     `;

//   compilation.hooks.runtimeRequirementInTree
//     .for(RuntimeGlobals.instantiateWasm)
//     .tap("FetchCompileAsyncWasmPlugin", (chunk, set) => {
//       const chunkGraph = compilation.chunkGraph;
//       if (
//         !chunkGraph.hasModuleInGraph(
//           chunk,
//           m => m.type === "webassembly/async-experimental"
//         )
//       ) {
//         return;
//       }
//       set.add(RuntimeGlobals.publicPath);
//       compilation.addRuntimeModule(
//         chunk,
//         new AsyncWasmChunkLoadingRuntimeModule(chunk, compilation, {
//           generateLoadBinaryCode,
//           supportsStreaming: true
//         })
//       );
//     });
// };

// compiler.outputFileSystem = fs;
compiler.run((err, stats) => {
  if (err) {
    throw err;
  }
  const { assets } = stats.compilation;
  const jsonStats = stats.toJson();
  const bundle = {
    wasm: null,
    script: "",
    // errors: jsonStats.errors
    // FIXME: changed in 5 to an Object
    errors: []
  };
  console.log(stats.toString());

  const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
  const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
  const hasWasmModule = wasmModuleAsset !== undefined;

  bundle.script = jsAssets.reduce((acc, k) => {
    const asset = assets[k];
    // FIXME: webpack 5 uses SourceOnlySize?
    // return acc + asset.source();
    return acc + readFileSync(join(fullConfig.output.path, k), "utf8");
  }, "");

  if (hasWasmModule === true) {
    bundle.wasm = Buffer.from(readFileSync(join(fullConfig.output.path, wasmModuleAsset))).toString();
    // FIXME: webpack 5 uses SourceOnlySize?
    // bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString();
  writeFileSync("./worker/module.wasm.from_dist", readFileSync(join(fullConfig.output.path, wasmModuleAsset)));
  }

  writeFileSync(args["output-file"], JSON.stringify(bundle));
});
