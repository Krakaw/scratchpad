const express = require("express");
const router = express.Router();
const axios = require("axios");
const {getGithubAuthHeaders} = require("../helpers/github");
const {GITHUB_API_RELEASE_BASE_URL} = process.env;

const deleteRemoteBranch = async function (req, res) {
    const url = `${GITHUB_API_RELEASE_BASE_URL}/git/refs/heads/${req.params.remoteBranch}`;
    const headers = getGithubAuthHeaders();
    await axios.delete(url, {headers});
    return res.json("Ok");
};

router.delete('/:remoteBranch', deleteRemoteBranch);
module.exports = router;
