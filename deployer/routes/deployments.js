const util = require("util");
const exec = util.promisify(require("child_process").exec);
const express = require("express");
const router = express.Router();
const { cleanBranch } = require("../helpers/branches");
const { getDirectories } = require("../helpers/instances");

const { RELEASES_DIR, RELEASE_BASE, DEBUG } = process.env;

const deployEndpoint = async function (req, res) {
    deployBranch(req.body.branch);
    return res.sendStatus(200);
};

async function deployFromQuay(req, res) {
    const branch = req.body.trigger_metadata.ref.split("/").pop();
    deployBranch(branch);
    return res.sendStatus(200);
}

async function deployBranch(branch, branchName) {
    branchName = (branchName || cleanBranch(branch)).toLowerCase();
    console.log("Starting deploy for", branch, "as", branchName);

    let resultText = `Deploying ${branch}`;
    const { stdout, stderr } = await exec(
        `cd ${RELEASE_BASE} && ./controller/create-scratch.sh -a '${branch}' -n '${branchName}'`
    );
    if (stderr) {
        console.error(resultText, "FAILED");
        if (DEBUG) {
            console.error(stderr);
        }
    } else {
        console.log(resultText, "SUCCESS");
        if (DEBUG) {
            console.log(stdout);
        }
    }
}

const clone = async function (req, res) {
    let dirs = getDirectories(RELEASES_DIR);
    let localBranch = cleanBranch(req.params.localBranch);
    let name = cleanBranch(
        `${localBranch}cloned${Math.floor(Math.random() * 10000 + 1)}`
    );
    if (req.body.name) {
        name = cleanBranch(req.body.name.toLowerCase());
    }
    if (
        dirs.indexOf(localBranch) === -1 ||
        !localBranch.trim() ||
        dirs.indexOf(name) > -1
    ) {
        return res.sendStatus(401);
    }
    await deployBranch(localBranch, name);
};

router.post("/", deployEndpoint);
router.post("/quay", deployFromQuay);
router.post("/:localBranch/clone", clone);


module.exports = router;
