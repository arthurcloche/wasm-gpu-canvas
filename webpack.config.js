const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require("copy-webpack-plugin");

module.exports = {
  entry: "./www/index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "./www/index.html",
    }),
    new webpack.ProvidePlugin({
      TextDecoder: ["text-encoding", "TextDecoder"],
      TextEncoder: ["text-encoding", "TextEncoder"],
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      extraArgs: "--target web",
    }),
    new CopyPlugin({
      patterns: [{ from: "examples", to: "examples" }],
    }),
  ],
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
  },
  resolve: {
    extensions: [".js", ".wasm"],
  },
  devServer: {
    static: [
      {
        directory: path.join(__dirname, "dist"),
      },
      {
        directory: path.join(__dirname, "examples"),
        publicPath: "/examples",
      },
    ],
    compress: true,
    port: 8080,
  },
};
