## build wasm lib

```
$ pwd
> ${PWD}/minimax_ttt/minimax_ttt

$ wasm-pack build --scope ${your npm user name}
```

optimize binary size

```
$ wasm-snip -o minimax_ttt.snipped.wasm ./target/wasm32-unknown-unknown/release/minimax_ttt.wasm
```

Let put optimized wasm to pkg dir and edit some config.

```
$ cp package.json ./pkg/package.json
```
