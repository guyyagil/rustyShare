let masterData = null;
let currentPath = "";

// Initialize from hash
if (window.location.hash.length > 1) {
  currentPath = decodeURIComponent(window.location.hash.slice(1));
}

// Upload logic
document.getElementById('uploadForm').onsubmit = async function(e) {
  e.preventDefault();
  const formData = new FormData(this);

  formData.append("target_path", currentPath); // 🟢 now currentPath is correctly passed
  const res = await fetch('/api/upload', {
    method: 'POST',
    body: formData
  });

  if (res.ok) {
    alert('Upload successful!');
    fetchmasterTree();
  } else {
    const err = await res.text();
    alert('Upload failed! ' + err);
  }
};

async function fetchmasterTree() {
  const res = await fetch("/api/master.json");
  masterData = await res.json();
  updateGrid();
}

function updateGrid() {
  console.log("Rendering path:", currentPath);
  const container = document.getElementById("tree");
  container.innerHTML = "";
  const search = document.getElementById("searchInput").value.trim().toLowerCase();
  const entry = findEntryByPath(masterData, currentPath);
  container.appendChild(renderGrid(entry, search));
  window.location.hash = encodeURIComponent(currentPath);
}

function renderGrid(entry, search = "") {
  if (!entry.is_dir || !entry.children) return document.createTextNode("No master found.");

  const grid = document.createElement("div");
  grid.className = "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-8";

  if (currentPath) {
    const backBtn = document.createElement("button");
    backBtn.textContent = "⬅️ Back";
    backBtn.className = "mb-4 px-4 py-2 bg-gray-300 rounded";
    backBtn.onclick = () => {
      const parent = currentPath.split("/").slice(0, -1).join("/");
      currentPath = parent;
      updateGrid();
    };
    grid.appendChild(backBtn);
  }

  for (const child of entry.children) {
    if (search && !child.name.toLowerCase().startsWith(search)) continue;

    const item = document.createElement("div");
    item.className =
      "flex flex-col items-center bg-gradient-to-br from-white via-blue-100 to-blue-200 rounded-2xl shadow-lg p-4 transition-transform duration-200 hover:scale-105 border border-blue-200";

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

    const label = document.createElement("div");
    label.className =
      "text-center text-blue-900 font-semibold text-base truncate w-full px-2 mb-2";
    label.title = child.name;
    label.textContent = child.name;

    item.appendChild(icon);
    item.appendChild(label);

    const btnGroup = document.createElement("div");
    btnGroup.className = "flex flex-col items-center gap-2 mt-3";

    if (child.is_dir || child.is_browser_supported) {
      const openBtn = document.createElement("button");
      openBtn.textContent = "Open";
      openBtn.className =
        "px-4 py-1 bg-blue-600 text-white rounded-full text-sm font-medium shadow hover:bg-blue-700 transition-all duration-150";
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

    if (!child.is_dir) {
      const downloadBtn = document.createElement("a");
      downloadBtn.textContent = "Download";
      downloadBtn.href = `/api/master/${encodeURIComponent(child.path)}`;
      downloadBtn.download = child.name;
      downloadBtn.className =
        "px-4 py-1 bg-blue-400 text-white rounded-full text-sm font-medium shadow hover:bg-blue-500 transition-all duration-150 inline-block text-center";
      btnGroup.appendChild(downloadBtn);

      const updateBtn = document.createElement("button");
      updateBtn.textContent = "Update";
      updateBtn.className =
        "px-4 py-1 bg-yellow-500 text-white rounded-full text-sm font-medium shadow hover:bg-yellow-600 transition-all duration-150";
      btnGroup.appendChild(updateBtn);

      const updateInput = document.createElement("input");
      updateInput.type = "file";
      updateInput.style.display = "none";
      updateInput.accept = "*/*";
      btnGroup.appendChild(updateInput);

      updateBtn.onclick = (e) => {
        e.stopPropagation();
        updateInput.click();
      };

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

function findEntryByPath(entry, path) {
  if (entry.path === path) return entry;
  if (!entry.children) return null;
  for (const child of entry.children) {
    const found = findEntryByPath(child, path);
    if (found) return found;
  }
  return null;
}

document.getElementById("searchInput").addEventListener("input", updateGrid);

fetchmasterTree();
setInterval(fetchmasterTree, 5000);