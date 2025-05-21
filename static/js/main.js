import { fetchMasterTree, uploadFile } from "./api.js";
import { renderGrid, findEntryByPath } from "./dom.js";

let masterData = null;
let currentPath = "";

if (window.location.hash.length > 1) {
  currentPath = decodeURIComponent(window.location.hash.slice(1));
}

function setPath(path) {
  currentPath = path;
  updateGrid();
}

async function refreshTree() {
  masterData = await fetchMasterTree();
  updateGrid();
}

function updateGrid() {
  const container = document.getElementById("tree");
  container.innerHTML = "";
  const search = document.getElementById("searchInput").value.trim().toLowerCase();
  const entry = findEntryByPath(masterData, currentPath);
  container.appendChild(renderGrid(entry, currentPath, setPath, refreshTree, search));
  window.location.hash = encodeURIComponent(currentPath);
}

document.getElementById("uploadForm").onsubmit = async function (e) {
  e.preventDefault();
  const formData = new FormData(this);
  const res = await uploadFile(formData, currentPath);
  alert(res.ok ? "Upload successful!" : "Upload failed! " + await res.text());
  refreshTree();
};

document.getElementById("searchInput").addEventListener("input", updateGrid);

refreshTree();
setInterval(refreshTree, 5000);
