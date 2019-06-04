const { parse } = require("@babel/parser");
const t = require("@babel/types");
const traverse = require("@babel/traverse").default;

const parserOpts = {
  sourceType: "module",
};

module.exports = function (content) {
  const { onKvStorageBinding } = this.query;
  const ast = parse(content, parserOpts);

  traverse(ast, {
    ImportDeclaration(path) {
      const bindingId = path.node.specifiers[0].local;
      const moduleSpecifier = path.node.source.value;

      if (moduleSpecifier.startsWith("cloudflare:kv-storage")) {
        const [, name] = moduleSpecifier.split("/");
        onKvStorageBinding(bindingId.name, name);

        path.remove();
      }
    }
  });

  return "";
}
