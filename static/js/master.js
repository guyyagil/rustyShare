// Manages the file tree state, current path, and handles navigation based on URL hash
let masterData = null;
let currentPath = "";

// If the URL has a hash, use it as the initial path
if (window.location.hash.length > 1) {
  currentPath = decodeURIComponent(window.location.hash.slice(1));
}

// Handles file uploads, sending the selected file and current path to the server
document.getElementById('uploadForm').onsubmit = async function(e) {
  e.preventDefault();
  const formData = new FormData(this);

  // Add the current path to the form data so the server knows where to upload
  formData.append("target_path", currentPath);
  const res = await fetch('/api/upload', {
    method: 'POST',
    body: formData
  });

  if (res.ok) {
    alert('Upload successful!');
    fetchmasterTree(); // Refresh the file tree after upload
  } else {
    const err = await res.text();
    alert('Upload failed! ' + err);
  }
};

// Fetches the latest file tree from the server and updates the UI
async function fetchmasterTree() {
  const res = await fetch("/api/master.json");
  masterData = await res.json();
  updateGrid();
}

// Renders the file/folder grid for the current path and search query
function updateGrid() {
  console.log("Rendering path:", currentPath);
  const container = document.getElementById("tree");
  container.innerHTML = "";
  const search = document.getElementById("searchInput").value.trim().toLowerCase();
  const entry = findEntryByPath(masterData, currentPath);
  container.appendChild(renderGrid(entry, search));
  window.location.hash = encodeURIComponent(currentPath); // Update URL hash
}

// Handles navigation, file/folder actions (open, download, update), and search filtering
function renderGrid(entry, search = "") {
  if (!entry.is_dir || !entry.children) return document.createTextNode("No master found.");

  const grid = document.createElement("div");
  grid.className = "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-8";

  // Add a back button if not at the root
  if (currentPath) {
    const backBtn = document.createElement("button");
    backBtn.className =
      "flex flex-col items-center justify-center mb-4 px-6 py-4 bg-gradient-to-br from-stone-100 via-stone-50 to-white border border-stone-200 shadow-lg rounded-xl transition-all duration-200 hover:shadow-xl hover:scale-105 cursor-pointer group";
    backBtn.onclick = () => {
      const parent = currentPath.split("/").slice(0, -1).join("/");
      currentPath = parent;
      updateGrid();
    };
    // Icon and label
    const icon = document.createElement("span");
    icon.className = "text-blue-400 text-3xl mb-2 group-hover:-translate-x-1 transition-transform duration-200";
    icon.innerHTML = "&#8592;"; // Left arrow
    const label = document.createElement("span");
    label.className = "text-blue-700 font-semibold";
    label.textContent = "Back";
    backBtn.appendChild(icon);
    backBtn.appendChild(label);
    grid.appendChild(backBtn);
  }

  // Loop through each child (file or folder) in the directory
  for (const child of entry.children) {
    // Filter by search query if present
    if (search && !child.name.toLowerCase().startsWith(search)) continue;

    // Create the visual item for the file/folder
    const item = document.createElement("div");
    item.className =
      "flex flex-col items-center bg-gradient-to-br from-stone-100 via-stone-50 to-white rounded-2xl shadow-lg p-4 transition-transform duration-200 hover:scale-105 border border-stone-200 relative group";

    // Tooltip for files (shows details on hover)
    if (!child.is_dir) {
      const tooltip = document.createElement("div");
      tooltip.className =
        "absolute z-10 left-1/2 -translate-x-1/2 bottom-full mb-2 w-64 bg-gray-900 text-white text-xs rounded-lg shadow-lg px-4 py-2 opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-200";
      tooltip.style.whiteSpace = "pre-line";
      tooltip.innerText =
        `Name: ${child.name}
        Size: ${child.size ? formatSize(child.size) : "?"}
        Modified: ${child.modified ? formatDate(child.modified) : "?"}`;
      item.appendChild(tooltip);
    }

    // Icon for the file/folder type
    const icon = document.createElement("div");
    icon.className =
      "text-6xl mb-3 transition-transform group-hover:scale-110 text-blue-400 drop-shadow";
    if (child.is_dir) {
      icon.textContent = "📂";
    } else if (child.file_type === "Audio") {
      icon.textContent = "🎵";
    } else if (child.file_type === "Image") {
      icon.textContent = "🖼️";
    } else if (child.file_type === "Video") {
      icon.textContent = "🎬";
    } else {
      icon.textContent = "📄";
    }

    // File/folder name label
    const label = document.createElement("div");
    label.className =
      "text-center text-blue-900 font-semibold text-base truncate w-full px-2 mb-2";
    label.title = child.name;
    label.textContent = child.name;

    item.appendChild(icon);
    item.appendChild(label);

    // Button group for actions (open, download, update)
    const btnGroup = document.createElement("div");
    btnGroup.className = "flex flex-col items-center gap-2 mt-3";

    // "Open" button for folders and browser-supported files
    if (child.is_dir || child.is_browser_supported) {
      const openBtn = document.createElement("button");
      openBtn.textContent = "Open";
      openBtn.className =
        "px-4 py-1 bg-emerald-500 text-white rounded-full text-sm font-medium shadow hover:bg-emerald-600 transition-all duration-150";
      openBtn.onclick = (e) => {
        e.stopPropagation();
        if (child.is_dir) {
          currentPath = child.path;
          updateGrid();
        } else {
          window.open(`/api/master/${encodeURIComponent(child.path)}`, "_blank");
        }
      };
      btnGroup.appendChild(openBtn);
    }

    // Download and update buttons for files
    if (!child.is_dir) {
      // Download button
      const downloadBtn = document.createElement("a");
      downloadBtn.textContent = "Download";
      downloadBtn.href = `/api/master/${encodeURIComponent(child.path)}`;
      downloadBtn.download = child.name;
      downloadBtn.className =
        "px-4 py-1 bg-blue-500 text-white rounded-full text-sm font-medium shadow hover:bg-blue-600 transition-all duration-150 inline-block text-center";
      btnGroup.appendChild(downloadBtn);

      // Update button and hidden file input
      const updateBtn = document.createElement("button");
      updateBtn.textContent = "Update";
      updateBtn.className ="px-4 py-1 bg-amber-200 text-amber-900 rounded-full text-sm font-medium shadow hover:bg-amber-300 transition-all duration-150";
      btnGroup.appendChild(updateBtn);

      const updateInput = document.createElement("input");
      updateInput.type = "file";
      updateInput.style.display = "none";
      updateInput.accept = "*/*";
      btnGroup.appendChild(updateInput);

      // When "Update" is clicked, trigger the file input
      updateBtn.onclick = (e) => {
        e.stopPropagation();
        updateInput.click();
      };

      // When a file is selected, upload it to replace the current file
      updateInput.onchange = async function () {
        if (!updateInput.files.length) return;
        const file = updateInput.files[0];
        const formData = new FormData();
        formData.append("file", file);
        formData.append("replace_path", child.path);

        const res = await fetch("/api/update", {
          method: "POST",
          body: formData,
        });
        if (res.ok) {
          alert("File updated!");
          fetchmasterTree();
        } else {
          const err = await res.text();
          alert("Update failed! " + err);
        }
        updateInput.value = "";
      };
    }

    item.appendChild(btnGroup);
    grid.appendChild(item);
  }

  return grid;
}

// Finds a file or folder entry in the tree by its path
function findEntryByPath(entry, path) {
  if (entry.path === path) return entry;
  if (!entry.children) return null;
  for (const child of entry.children) {
    const found = findEntryByPath(child, path);
    if (found) return found;
  }
  return null;
}

// Listens for search input changes and updates the grid
document.getElementById("searchInput").addEventListener("input", updateGrid);

// Initializes the file tree and sets up periodic refresh
fetchmasterTree();
setInterval(fetchmasterTree, 5000);

// Formats file sizes and dates for display
function formatSize(bytes) {
  if (!bytes) return "?";
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (bytes >= 1024 && i < sizes.length - 1) {
    bytes /= 1024;
    i++;
  }
  return bytes.toFixed(1) + " " + sizes[i];
}

function formatDate(ts) {
  // If ts is a string, try to parse it
  const d = new Date(ts);
  if (isNaN(d)) return "?";
  return d.toLocaleString();
}