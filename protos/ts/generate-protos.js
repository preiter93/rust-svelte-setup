import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { join, resolve, dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PROTO_SRC = join(__dirname, "../*.proto");
const OUT_DIR = resolve(__dirname, "../../app/src/lib/protos");

execSync(`proto-loader-gen-types \
  --longs=String \
  --keepCase \
  --defaults \
  --oneofs \
  --grpcLib=@grpc/grpc-js \
  --outDir=${OUT_DIR} \
  ${PROTO_SRC}`, { stdio: "inherit" });
