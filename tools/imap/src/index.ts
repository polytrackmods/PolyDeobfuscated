#!/usr/bin/env node

import * as fs from "fs";
import * as path from "path";

import { Command } from "commander";

const program = new Command();

program
    .name("imap")
    .description("A tool for creating source maps for deobfuscated files")
    .version("1.0.0");

program
    .command("create <file> <line> <original> <new>")
    .description("Create a source map for a deobfuscated file")
    .action((file, line, original, new_) => {
        const filePath = path.resolve(file);
        const lineNumber = parseInt(line, 10);

        const sourceMap = {
            file: filePath,
            line: lineNumber,
            original,
            new: new_,
        };

        const sourceMapDir = path.join(process.cwd(), "source_maps");

        if (!fs.existsSync(sourceMapDir)) {
            fs.mkdirSync(sourceMapDir, { recursive: true });
        }

        const sourceMapFilePath = path.join(
            sourceMapDir,
            `${path.basename(filePath)}.json`
        );

        if (fs.existsSync(sourceMapFilePath)) {
            const existingSourceMap = JSON.parse(
                fs.readFileSync(sourceMapFilePath, "utf-8")
            );
            existingSourceMap.push(sourceMap);
            fs.writeFileSync(
                sourceMapFilePath,
                JSON.stringify(existingSourceMap, null, 4)
            );
        } else {
            fs.writeFileSync(
                sourceMapFilePath,
                JSON.stringify([sourceMap], null, 4)
            );
        }
    });

program.parse();
