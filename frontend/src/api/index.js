const serverUrl = process.env.REACT_APP_SERVER_URL || 'http://localhost:3000'

export const getInstances = async () => {
    return await (await fetch(`${serverUrl}/instances`)).json()
}

export const getRepos = async () => {
    return await (await fetch(`${serverUrl}/github/repos`)).json();
}

export const getWorkflows = async (owner, repo) => {
    const url = encodeURI(`${serverUrl}/github/workflows?owner=${owner}&repo=${repo}`);
    return await (await fetch(url)).json();
}

export const getPackages = async (owner, repo) => {
    const url = encodeURI(`${serverUrl}/github/packages?owner=${owner}&repo=${repo}`);
    return await (await fetch(url)).json();
}

export const getBranches = async (owner, repo) => {
    const url = encodeURI(`${serverUrl}/github/branches?owner=${owner}&repo=${repo}`);
    return await (await fetch(url)).json();
}

export const getPrs = async (owner, repo) => {
    const url = encodeURI(`${serverUrl}/github/prs?owner=${owner}&repo=${repo}`);
    return await (await fetch(url)).json();
}

export const deletePackage = async (id) => {
    const url = encodeURI(`${serverUrl}/github/package/${id}`);
    return await fetch(url, {method: 'DELETE'});
}
