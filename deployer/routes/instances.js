const fs = require("fs");
const util = require("util");
const exec = util.promisify(require("child_process").exec);
const express = require("express");
const router = express.Router();
const {getDirStats, getDockerStatus, readInstanceConfig, readInstanceVersions, getDirectories, isValidLocalBranch} = require("../helpers/instances");
const {getBranches, getGithubAuthHeaders, getPullRequestDetails} = require("../helpers/github");
const {cleanBranch} = require("../helpers/branches");
const {API_RELEASE_BRANCHES_URL, API_BRANCHES_URL, GITHUB_WEB_BRANCHES_URL, API_PULL_REQUEST_URL, RELEASES_DIR, DEBUG} = process.env;

const branches = async function (req, res) {
    let dirs = getDirectories(RELEASES_DIR);
    let dirStats = getDirStats(dirs);
    let dockerStatus = await getDockerStatus();
    const headers = getGithubAuthHeaders();
    let webBranches = await getBranches(GITHUB_WEB_BRANCHES_URL, headers);
    let apiReleaseRemoteBranches = await getBranches(
        API_RELEASE_BRANCHES_URL,
        headers
    );
    let apiRemoteBranches = await getBranches(API_BRANCHES_URL, headers);
    let apiPullRequests = await getBranches(API_PULL_REQUEST_URL, headers);
    let pullRequestDetails = await getPullRequestDetails(
        API_PULL_REQUEST_URL,
        headers
    );
    apiRemoteBranches = [...new Set(apiRemoteBranches.concat(apiPullRequests))];

    let usedDirs = [];
    /** Check all of the remote branches from the api and see if we have a local equivalent*/
    let apiReleaseBranches = apiReleaseRemoteBranches.map(branch => {
        let localBranch = cleanBranch(branch);
        let {birthtimeMs: createdAt = 0} = dirStats[branch] || {};
        usedDirs.push(localBranch);
        return {
            hasRemote: true,
            remote: branch,
            local: localBranch,
            exists: dirs.indexOf(localBranch) > -1,
            existsOnSourceRepo: apiRemoteBranches.indexOf(branch) > -1,
            ports: {},
            createdAt,
            extra: pullRequestDetails[branch] || {},
            dockerStatus: dockerStatus[localBranch] || [],
            versions: {}
        };
    });

    /** Check that all of our local instances have been accounted for and return any that didn't have a remote branch */
    dirs.forEach(dir => {
        const convertedApiRemoteBranches = apiRemoteBranches.map(cleanBranch);
        if (usedDirs.indexOf(dir) === -1) {
            let {birthtimeMs: createdAt = 0} = dirStats[dir] || {};
            apiReleaseBranches.push({
                hasRemote: false,
                remote: `${dir}`,
                local: dir,
                createdAt,
                exists: true,
                existsOnSourceRepo: convertedApiRemoteBranches.indexOf(dir) > -1,
                ports: {},
                dockerStatus: dockerStatus[dir] || [],
                versions: {}
            });
        }
    });

    /** Add instance specific data to each local instance */
    for (let i in apiReleaseBranches) {
        const releaseBranch = apiReleaseBranches[i];
        if (!releaseBranch.exists) {
            continue;
        }
        let config = {};
        let versions = {};
        try {
            config = readInstanceConfig(releaseBranch.local);
        } catch (e) {
            console.error("Failed to read config for", releaseBranch.local);
        }

        try {
            versions = await readInstanceVersions(releaseBranch.local);
        } catch (e) {
            console.error("Failed to read versions for", releaseBranch.local);
        }

        releaseBranch.versions = versions;
    }
    return res.json({api: apiReleaseBranches, web: webBranches});
};

const logs = async function (req, res) {
    let localBranch = cleanBranch(req.params.localBranch);
    if (!isValidLocalBranch(req.params.localBranch)) {
        return 401;
    }

    let file = fs.readFileSync(
        `${RELEASES_DIR}/${localBranch}/web/build/server.log`
    );
    return res.send(file);
};

const executeInstanceScript = async function (localBranchParam, script, params = [], returnResult = false) {
    let localBranch = cleanBranch(localBranchParam);
    console.log(`Executing: "${script} ${params.join(" ")}" on ${localBranchParam}`);
    if (!isValidLocalBranch(localBranchParam)) {
        return 401;
    }
    const result = await exec(`cd ${RELEASES_DIR}/${localBranch} && ./${script} ${params.join(" ")}`);
    return returnResult ? result : 200;
};

const linkToWeb = async function (req, res) {
    let webBranch = req.body.webBranch;
    return res.sendStatus(await executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--web", `'${webBranch}'`, '&']));
};

const stop = async function (req, res) {
    return res.sendStatus(await executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--stop"]));
};

const start = async function (req, res) {
    return res.sendStatus(await executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--start"]));
};

const wipeDb = async function (req, res) {
    return res.sendStatus(await executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--wipe"]));
};

const deleteScratch = async function (req, res) {
    return res.sendStatus(await executeInstanceScript(req.params.localBranch, "delete.sh"));
};

const getEnv = async function (req, res) {
    const {stdout: env, stderr} = await executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--env"], true);
    const [api, web, cube] = env.split("|--|").map(s => s.trim());
    return res.json({api, web, cube});
};

const setEnv = async function(req, res) {
    const {api, web, cube} = req.body;
    const localBranch = req.params.localBranch;
    if (!isValidLocalBranch(localBranch)) {
        return 401;
    }

    try {
        const path = `${RELEASES_DIR}/${localBranch}`;
        fs.writeFileSync(`${path}/.api.env`, api);
        fs.writeFileSync(`${path}/.web.env`, web);
        fs.writeFileSync(`${path}/.bn-cube.env`, cube);
        executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--rebuild"], true);
    } catch(err) {
        console.error(err);
        // An error occurred
        return res.sendStatus(500);
    }
    return res.sendStatus(200);
};

const resetEnv = async function (req, res) {
    const {envs} = req.body;
    const localBranch = req.params.localBranch;
    if (!isValidLocalBranch(localBranch)) {
        return 401;
    }
    const validEnvs = ['api', 'web', 'bn-cube'];
    envs.forEach(env => {
        if (!validEnvs.find(e => e===env)) {
            return;
        }
        executeInstanceScript(req.params.localBranch, "manage-instance.sh", ["--reset-env", env], true);
    });
    return res.sendStatus(200);
};
router.get('/', branches);
router.get('/:localBranch/logs', logs);
router.get('/:localBranch/env', getEnv);
router.put('/:localBranch/env', setEnv);
router.post('/:localBranch/env', resetEnv);
router.post('/:localBranch/web', linkToWeb);
router.post('/:localBranch/stop', stop);
router.post('/:localBranch/start', start);
router.delete('/:localBranch', deleteScratch);
router.delete('/:localBranch/db', wipeDb);

module.exports = router;
