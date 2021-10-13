const express = require("express");
const router = express.Router();
const {deletePackage, getPackages, dispatchWorkflow} = require('../helpers/github');
const {GITHUB_GRAPHQL_PACKAGES_WEB, GITHUB_GRAPHQL_PACKAGES_API} = process.env;
const deleteImage = async function (req, res) {
    try {
        const {source, id} = req.params;
        switch (source.toLowerCase()) {
            case 'github_packages':
                let result = await deletePackage(id);
                return res.json(result);
                break;
            default:
                return res.sendStatus(404).json({error: "Invalid source"})
        }
    } catch (e) {
        console.error(e);
        res.sendStatus(500)
    }
}

const getImages = async function (req, res) {
    const [api_owner, _api_package, ...apiPackageName] = GITHUB_GRAPHQL_PACKAGES_API.split('/');
    const api = await getPackages(api_owner, apiPackageName.join('/'));
    const [web_owner, _web_package, ...webPackageName] = GITHUB_GRAPHQL_PACKAGES_WEB.split('/');
    const web = await getPackages(web_owner, webPackageName.join('/'));

    return res.json({api, web})
}

const getBuildFunction = (source) => {
    switch (source) {
        case 'github_packages':
            return dispatchWorkflow;
        default:
            return function() {throw `Builder not available for ${source}`}
    }
}

const buildImage = async function (req, res) {
    try {
        const {branch, stack, source} = req.body;
        const buildFunction = getBuildFunction(source);
        let owner, package;
        switch (stack) {
            case 'api':
                [owner, package, workflowId] = GITHUB_GRAPHQL_PACKAGES_API.split('/');

                break;
            case 'web':
                [owner, package, workflowId] = GITHUB_GRAPHQL_PACKAGES_WEB.split('/');
                break;
        }
        const result = await buildFunction(owner, package, workflowId, branch)
        return res.json(result);

    } catch (e) {
        console.error(e);
        res.sendStatus(500)
    }
}
router.get("/:source", getImages);
router.post("/build", buildImage)
router.delete("/:source/:id", deleteImage)

module.exports = router;
