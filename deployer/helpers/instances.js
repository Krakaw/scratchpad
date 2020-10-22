const fs = require("fs");
const util = require("util");
const exec = util.promisify(require("child_process").exec);
const {cleanBranch} = require("./branches");
const {RELEASES_DIR} = process.env;

function isValidLocalPath(branch, file) {
    return isValidLocalBranch(branch) && fs.existsSync(`${RELEASES_DIR}/${cleanBranch(branch)}/${file}`);
}

function isValidLocalBranch(branch) {
    const dirs = getDirectories(RELEASES_DIR);
    const localBranch = cleanBranch(branch);
    return !(dirs.indexOf(localBranch) === -1 || !localBranch.trim());
}

function getDirectories(dir) {
    return fs.readdirSync(dir).filter(d => fs.lstatSync(`${dir}/${d}`).isDirectory())
}

function getDirStats(dirs) {
    const stats = {};
    dirs.forEach(dir => {
        let stat = fs.statSync(`${RELEASES_DIR}/${dir}`);
        stats[dir] = stat;
    });
    return stats;
}

function readInstanceConfig(dir) {
    let config = {};
    try {
        const filePath = `${RELEASES_DIR}/${dir}/docker-source.sh`;
        if (fs.existsSync(filePath))
        {
            let raw = fs.readFileSync(filePath, "utf8");
            let rows = raw
                .trim()
                .replace(/^\s*$/gim, "")
                .replace(/export /gim, "")
                .split("\n")
                .map(r => {
                    let splitPos = r.indexOf("=");
                    let result = [r.substr(0, splitPos), r.substr(splitPos + 1)];
                    return result;
                })
                .filter(r => r.length > 1);

            rows.forEach(row => {
                config[row[0]] = row[1];
            });
        }

    } catch (e) {
        console.error("Failed to readInstanceConfig", dir, e);
    }
    return config;
}

async function readInstanceVersions(dir) {
    let versions = {api: "?", web: "?"};
    try {
        const filePath = `${RELEASES_DIR}/${dir}/web/build/version.json`;
        if (fs.existsSync(filePath)) {
            let raw = fs.readFileSync(filePath, "utf8");
            let json = JSON.parse(raw);
            versions.web = json.version || json;
        }

    } catch (e) {
        console.error("Failed to readInstanceVersions for web", dir, e);
    }
    try {
        const filePath = `${RELEASES_DIR}/${dir}/api_version.txt`;
        if (fs.existsSync(filePath)) {
            versions.api = fs.readFileSync(filePath, "utf8");
        }
    } catch (e) {
        console.error("Failed to readInstanceVersions for api", dir, e);
    }
    return versions;
}

async function getDockerStatus() {
    const cmd = `docker ps -a --format '{{.Names}}|{{.CreatedAt}}|{{.Image}}|{{.Status}}'`;
    const {stdout, stderr} = await exec(cmd);
    const statusResult = {};
    stdout.split(`\n`).forEach(line => {
        const [name, createdAt, image, status] = line.split("|");
        const [key] = name.split("_");
        if (!statusResult.hasOwnProperty(key)) {
            statusResult[key] = [];
        }
        statusResult[key].push({name, createdAt, image, status});
    });
    return statusResult;
}

module.exports = {
    readInstanceConfig,
    readInstanceVersions,
    getDirStats,
    getDockerStatus,
    getDirectories,
    isValidLocalBranch,
    isValidLocalPath
};
