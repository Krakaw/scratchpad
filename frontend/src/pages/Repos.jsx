import React, {useEffect, useState} from 'react'
import {getPackages, getRepos, getWorkflows,deletePackage, getBranches} from "../api";

function Repos() {
    const [selectedRepo, setSelectedRepo] = useState();
    const [repos, setRepos] = useState([])
    const [packages, setPackages] = useState([]);
    const [branches, setBranches] = useState([]);
    const [workflows, setWorkflows] = useState([]);

    useEffect(() => {
        getRepos().then(repos => {
            setRepos(repos);
        })
    }, [])

    const selectRepo = (repoName) => {
        setSelectedRepo(repoName);
        const [owner, repo] = repoName.split('/');
        getWorkflows(owner,repo).then(workflowResponse => {
            const workflows = workflowResponse.workflows;
            workflows.sort((a,b) => a.name > b.name ? 1 : b.name > a.name ? -1 : 0)
            setWorkflows(workflows);
        });
        getPackages(owner, repo).then(packageResponse => {
            packageResponse.sort((a,b) => a.version > b.version ? 1 : b.version > a.version ? -1 : 0)
            setPackages(packageResponse)
        });
        getBranches(owner, repo).then(branchesResponse => {
            branchesResponse.sort()
            setBranches(branchesResponse);
        })

    }

    const deleteRepoPackage = async (id) => {
        await deletePackage(id);
        packages.splice(packages.findIndex(p => p.id === id), 1)
        setPackages([...packages]);
    }

    return (
        <div className="row">
            <div className="col-sm-3">
                <h3>Repositories</h3>
                {repos.map(repo => <div key={repo} className="list-group">
                    <a href="#" onClick={() => selectRepo(repo)}
                       className="list-group-item list-group-item-action"
                       key={repo}>{repo}</a>
                </div>)}
            </div>
            <div className="col-sm-9">
                <div>
                    <h3>Branches</h3>
                    <ul className="list-group">
                        {branches.map(branch => <li style={{display: 'flex'}} key={branch} className="list-group-item">
                            {branch}
                            <span style={{flex:1}}></span>
                        </li>)}
                    </ul>
                </div>
                <div>
                    <h3>Packages</h3>
                    <ul className="list-group">
                        {packages.map(packageItem => <li style={{display: 'flex'}} key={packageItem.id} className="list-group-item">
                            {packageItem.version}
                            <span style={{flex:1}}></span>
                            <button className={'btn'}  onClick={() => {deleteRepoPackage(packageItem.id)}}>❌</button>
                        </li>)}
                    </ul>
                </div>
                <div>
                    <h3>Workflows</h3>
                    <ul className="list-group">
                        {workflows.map(workflow => <li style={{display: 'flex'}} key={workflow.id} className="list-group-item">
                            <a href={workflow.html_url} target="_blank">{workflow.name}</a>
                        </li>)}
                    </ul>
                </div>
            </div>
        </div>
    )
}

export default Repos;
