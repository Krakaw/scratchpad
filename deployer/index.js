require("dotenv").config();
const express = require("express");
const cors = require("cors");

//----Custom const
const PORT = process.env.PORT || 3000;
const {GITHUB_API_BASE_URL, GITHUB_WEB_BASE_URL, RELEASE_BASE} = process.env;
process.env.RELEASES_DIR = `${RELEASE_BASE}/releases`;
process.env.GITHUB_WEB_BRANCHES_URL = `${GITHUB_WEB_BASE_URL}/branches?per_page=1000`;
process.env.API_BRANCHES_URL = `${GITHUB_API_BASE_URL}/branches?per_page=1000`;
process.env.API_PULL_REQUEST_URL = `${GITHUB_API_BASE_URL}/pulls?per_page=1000`;
//----End custom const

const app = express();

app.use(cors());
app.use(express.json());
app.use(express.urlencoded({extended: true}));

app.get("/healthcheck", function (req, res) {
    console.log("Still running!");
    res.send("OK");
});

app.use('/scratches', require('./routes/deployments'));
app.use('/instances', require('./routes/instances'));
app.use('/github', require('./routes/github'));
app.use('/controller', require('./routes/controller'));

app.listen(PORT, () => console.log(`Scratch controller listening on port: ${PORT}`));
