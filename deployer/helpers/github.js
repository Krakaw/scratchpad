const axios = require("axios");
const cache = require('./cache');
const {graphql} = require("@octokit/graphql");
const {Octokit} = require("@octokit/rest");
const {GITHUB_USER_AGENT, GITHUB_PERSONAL_ACCESS_TOKEN, DEBUG} = process.env;

const graphqlWithAuth = graphql.defaults({
    headers: {
        authorization: `token ${GITHUB_PERSONAL_ACCESS_TOKEN}`
    }
})

function getGithubAuthHeaders() {
    return {
        "Content-Type": "application/json",
        "User-Agent": GITHUB_USER_AGENT,
        Authorization: `Basic ${Buffer.from(GITHUB_USER_AGENT + ":" + GITHUB_PERSONAL_ACCESS_TOKEN).toString("base64")}`
    };
}

async function getPackageDetailsForOrg(owner) {
    const octokit = new Octokit({
        auth: GITHUB_PERSONAL_ACCESS_TOKEN,
    });

    const r = await octokit.request('GET https://api.github.com/orgs/{owner}/packages', {
        owner,
        package_type: 'container',

    });
    return r;
}

async function getPackageVersions(owner, name, offset = 1, limit = 100) {
    const octokit = new Octokit({
        auth: GITHUB_PERSONAL_ACCESS_TOKEN,
    });

    const r = await octokit.request('GET https://api.github.com/orgs/{owner}/packages/container/{name}/versions', {
        name,
        owner,
        per_page: limit,
        page: offset
    });
    const next = r.headers.link.match(/\<https:\/\/.*page=(\d+)\>; rel="next"/);
    let nextPage;
    if (next && next[1]) {
        nextPage = +next[1];
    }
    const items = r.data.map(d => {
        const {metadata = {container: {}}} = d;
        const {tags = []} = metadata.container;
        return {
            id: d.id,
            name: tags.join(' ')
        };
    })
    let pageInfo = {nextPage};
    return [pageInfo, items];
}

async function getPackages(owner, name) {
    let result = [];
    let pageInfo = {nextPage:1};
    while (pageInfo.nextPage) {
        const offset = pageInfo.nextPage;
        const [returnedPageInfo, items] = await getPackageVersions(owner, name, offset);
        pageInfo = returnedPageInfo;
        result = result.concat(items)
    }
    return result;
}

async function deletePackage(packageId) {
    let graphqlWithAuth = graphql.defaults({
        headers: {
            accept: `application/vnd.github.package-deletes-preview+json`,
            authorization: `token ${GITHUB_PERSONAL_ACCESS_TOKEN}`
        },
        method: 'POST'
    })
    const result = await graphqlWithAuth(`
       mutation {
            deletePackageVersion( input: { packageVersionId: "${packageId}" }) { success }            
       }
    `);
    return result;
}

async function dispatchWorkflow(owner, repo, workflow_id, branch) {
    const octokit = new Octokit({
        auth: GITHUB_PERSONAL_ACCESS_TOKEN,
    });
    return await octokit.actions.createWorkflowDispatch({
        owner,
        repo,
        workflow_id,
        ref: branch,
    });
}

async function getBranchNames(url, headers) {
    let rawData = [];
    if (cache.hasOwnProperty(url) && cache[url].expires > new Date().getTime()) {
        rawData = cache[url].data;
    } else {
        try {
            const axiosResponse = await axios.get(url, {headers});
            let {data} = axiosResponse;
            rawData = data;
            cache[url] = {
                expires: new Date().getTime() + 60000, //Cache for a minute
                data
            };
        } catch (e) {
            console.error(`Could not fetch ${url}`, e);
        }
    }
    let branchNames = rawData.map(i => i.name || i.head.ref);
    branchNames.sort();
    return branchNames;
}

async function getWorkflows(owner, repo) {
    const url = `https://api.github.com/repos/${owner}/${repo}/actions/workflows`;
    let {data} = await axios.get(url, {headers: getGithubAuthHeaders()});
    return data;

}

async function getPullRequestDetails(url, headers) {
    let branches = [];
    if (cache.hasOwnProperty(url) && cache[url].expires > new Date().getTime()) {
        branches = cache[url].data;
    } else {
        let {data} = await axios.get(url, {headers});
        branches = data;
    }
    let result = {};
    const urlRegexp = /\b((?:[a-z][\w-]+:(?:\/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}\/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'".,<>?«»“”‘’]))/gi;


    await Promise.all(branches.map(async branch => {
        //console.log(branch);
        let {
            head = {},
            labels = [],
            body = "",
            title = "",
            statuses_url = "",
            state = "",
            html_url: githubUrl = ""
        } = branch;
        let {data} = await axios.get(statuses_url, {headers});
        let {state: buildStatus, description: buildDescription, updated_at: lastBuild, target_url: buildUrl} = data.length > 0 ? data[0] : {};

        body = body || '';
        let {ref: branchName} = head;
        labels = labels.map(label => ({name: label.name, color: label.color}));
        let urls = (body.match(urlRegexp) || []).filter(
            url => typeof url === "string" && url.indexOf("asana") !== -1
        ).map(i => ({type: 'asana', url: i}));
        urls.splice(0, 0, {type: 'github', url: githubUrl});

        result[branchName] = {
            labels,
            state,
            githubUrl,
            branchName,
            urls,
            body,
            title, buildStatus, buildDescription, lastBuild, buildUrl
        };
    }));

    if (DEBUG) {
        console.log(result);
    }

    return result;
}

module.exports = {
    getPackages,
    deletePackage,
    getWorkflows,
    getBranchNames,
    getPullRequestDetails,
    getGithubAuthHeaders,
    dispatchWorkflow

};
