import { execSync } from "child_process";
import { fileURLToPath } from "url";
import { join, resolve, dirname } from "path";
import { readdirSync, statSync } from "fs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const servicesRoot = resolve(__dirname, "../../services");

const services = readdirSync(servicesRoot)
  .filter(name => {
    const dir = join(servicesRoot, name);
    if (!statSync(dir).isDirectory()) return false;

    const files = readdirSync(dir);
    return files.some(f => f.endsWith(".proto"));
  });

services.forEach(service => {
  const PROTO_SRC = join(__dirname, `../../services/${service}/*proto`);
  const OUT_DIR = resolve(__dirname, `../../app/src/lib/protos/${service}`);

  execSync(`proto-loader-gen-types \
    --longs=String \
    --keepCase \
    --defaults \
    --oneofs \
    --grpcLib=@grpc/grpc-js \
    --outDir=${OUT_DIR} \
    ${PROTO_SRC}`, { stdio: "inherit" });
});
