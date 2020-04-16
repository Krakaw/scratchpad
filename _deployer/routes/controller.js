const util = require("util");
const exec = util.promisify(require("child_process").exec);
const express = require("express");
const router = express.Router();
const {RELEASE_BASE, DEBUG} = process.env;

const start = async function(req, res) {
    const {stdout, stderr} = await exec(
        `cd ${RELEASE_BASE}/controller && docker-compose up -d`
    );
    if (stderr) {
        if (DEBUG) {
            console.error(stderr);
        }
    } else {
        if (DEBUG) {
            console.log(stdout);
        }
    }
    return res.sendStatus(200);
};
router.post('/start', start);
module.exports = router;
