function cleanBranch(branch) {
    return (branch || '').replace(/[^a-z0-9\-_]/gi, "");
}

module.exports = {
    cleanBranch
};
