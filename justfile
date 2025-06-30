[working-directory: 'services']
generate-protos-rs:
  for d in */;\
    do echo "Generating protos in $d"; \
    just -f $d/justfile generate-protos; \
  done

[working-directory: 'app']
generate-protos-ts:
  just -f ./justfile generate-protos

generate-protos: generate-protos-rs generate-protos-ts
