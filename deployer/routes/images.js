const express = require("express");
const router = express.Router();
const {deletePackage, getPackages} = require('../helpers/github');
const {   GITHUB_GRAPHQL_PACKAGES_WEB, GITHUB_GRAPHQL_PACKAGES_API} = process.env;
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

const getImages = async function(req,res) {
    const [api_owner, api_package] = GITHUB_GRAPHQL_PACKAGES_API.split('/');
    const api = await getPackages(api_owner, api_package);
    const [web_owner, web_package] = GITHUB_GRAPHQL_PACKAGES_WEB.split('/');
    const web = await getPackages(web_owner, web_package);

    return res.json({api, web})
}
router.get("/:source", getImages);
router.delete("/:source/:id", deleteImage)

module.exports = router;
