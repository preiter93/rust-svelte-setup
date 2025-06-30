[working-directory: 'protos/rs']
generate-protos-rs:
  cargo run

[working-directory: 'protos/ts']
generate-protos-ts:
  npm install
  npm run proto

generate-protos: generate-protos-rs generate-protos-ts
