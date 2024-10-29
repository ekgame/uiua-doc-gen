import { Command } from "https://deno.land/x/cliffy@v1.0.0-rc.4/command/mod.ts";
import { resolve } from "jsr:@std/path";

async function getUiuaLibraryAsJson(directory: string) {
  const extractorDirectory = resolve(import.meta.dirname!, "../extractor/Cargo.toml");
  const command = new Deno.Command("cargo", {
    args: ['run', `--manifest-path=${extractorDirectory}`, directory],
    stdout: "piped",
  });
  const process = command.spawn();
  const status = await process.status;
  if (!status.success) {
    throw new Error("Failed to extract Uiua library information.");
  }
  const decoder = new TextDecoder();
  const output = await process.output();
  return JSON.parse(decoder.decode(output.stdout));
}

await new Command()
  .name("uiua-docs-gen")
  .version("0.1.0")
  .description("Documentation generator for Uiua.")
  .option("-d, --directory <directory:string>", "The directory to generate the documentation for.", { default: "." })
  .action(async (options) => {
    const directory = resolve(options.directory);
    console.log(`Generating documentation for ${directory}...`);
    const jsonData = await getUiuaLibraryAsJson(directory);
  })
  .parse(Deno.args);

