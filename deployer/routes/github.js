const express = require("express");
const router = express.Router();
const axios = require("axios");
const {getGithubAuthHeaders, getWorkflows, getPackages, deletePackage} = require("../helpers/github");
const {GITHUB_API_RELEASE_BASE_URL, GITHUB_REPOS} = process.env;

const deleteRemoteBranch = async function (req, res) {
    const url = `${GITHUB_API_RELEASE_BASE_URL}/git/refs/heads/${req.params.remoteBranch}`;
    const headers = getGithubAuthHeaders();
    await axios.delete(url, {headers});
    return res.json("Ok");
};

const getRepoPackages = async function(req, res) {
    const {owner, repo} = req.query;
    const data = await getPackages(owner, repo);
    return res.json(data);
}

const deleteRepoPackage = async function(req, res) {
    const {id} = req.params;
    await deletePackage(id);
    return res.sendStatus(200);
}

const getRepoWorkflows = async function(req, res) {
    const {owner, repo} = req.query;
    const data = await getWorkflows(owner, repo);
    return res.json(data);
}
const getRepos = async function(req,res) {
    return res.json(GITHUB_REPOS.split('|'));
}

router.delete('/:remoteBranch', deleteRemoteBranch);
router.get('/repos', getRepos);
router.get('/workflows', getRepoWorkflows);
router.get('/packages', getRepoPackages);
router.delete('/package/:id', deleteRepoPackage);
module.exports = router;
