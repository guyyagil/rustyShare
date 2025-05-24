import { fetchMasterTree, uploadFile, createFolder } from "./api.js";
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

// Add event listeners for the integrated buttons
document.getElementById("uploadBtn").addEventListener("click", () => {
  const uploadForm = document.getElementById("uploadForm");
  const createForm = document.getElementById("createFolderForm");
  
  // Toggle upload form and hide create form
  if (uploadForm.style.display === "none" || uploadForm.style.display === "") {
    uploadForm.style.display = "flex";
    createForm.style.display = "none";
  } else {
    uploadForm.style.display = "none";
  }
});

document.getElementById("createFolderBtn").addEventListener("click", () => {
  const uploadForm = document.getElementById("uploadForm");
  const createForm = document.getElementById("createFolderForm");
  
  // Toggle create form and hide upload form
  if (createForm.style.display === "none" || createForm.style.display === "") {
    createForm.style.display = "flex";
    uploadForm.style.display = "none";
  } else {
    createForm.style.display = "none";
  }
});

document.addEventListener("DOMContentLoaded", () => {
  const createFolderForm = document.getElementById("createFolderForm");
  if (createFolderForm) {
    createFolderForm.onsubmit = async (e) => {
      e.preventDefault();
      const input = document.getElementById("newFolderName");
      const folderName = input.value.trim();
      if (!folderName) return;
      // Use the module-scoped currentPath variable
      const path = currentPath ? `${currentPath}/${folderName}` : folderName;
      const res = await createFolder(path);
      if (res.ok) {
        alert("Folder created!");
        input.value = "";
        refreshTree();
      } else {
        alert("Failed to create folder: " + await res.text());
      }
    };
  }
});

refreshTree();
setInterval(refreshTree, 5000);
