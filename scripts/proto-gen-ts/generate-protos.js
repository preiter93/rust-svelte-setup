import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { join, resolve, dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// TODO: Do this for all services
const PROTO_SRC = join(__dirname, "../../services/users/*proto");
const OUT_DIR = resolve(__dirname, "../../app/src/lib/protos/users");

execSync(`proto-loader-gen-types \
  --longs=String \
  --keepCase \
  --defaults \
  --oneofs \
  --grpcLib=@grpc/grpc-js \
  --outDir=${OUT_DIR} \
  ${PROTO_SRC}`, { stdio: "inherit" });
