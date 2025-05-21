// --- dom.js ---

import { updateFile, deleteFile } from "./api.js";

export function formatSize(bytes) {
  if (!bytes) return "?";
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (bytes >= 1024 && i < sizes.length - 1) {
    bytes /= 1024;
    i++;
  }
  return bytes.toFixed(1) + " " + sizes[i];
}

export function formatDate(ts) {
  const d = new Date(ts);
  if (isNaN(d)) return "?";
  return d.toLocaleString();
}

export function findEntryByPath(entry, path) {
  if (entry.path === path) return entry;
  if (!entry.children) return null;
  for (const child of entry.children) {
    const found = findEntryByPath(child, path);
    if (found) return found;
  }
  return null;
}

export function renderGrid(entry, currentPath, setPath, refreshTree, search = "") {
  if (!entry?.is_dir || !entry.children) return document.createTextNode("No master found.");

  const grid = document.createElement("div");
  grid.className = "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-8";

  if (currentPath) {
    const backBtn = document.createElement("button");
    backBtn.className =
      "flex flex-col items-center justify-center mb-4 px-6 py-4 bg-gradient-to-br from-stone-100 via-stone-50 to-white border border-stone-200 shadow-lg rounded-xl transition-all duration-200 hover:shadow-xl hover:scale-105 cursor-pointer group";
    backBtn.onclick = () => {
      const parent = currentPath.split("/").slice(0, -1).join("/");
      setPath(parent);
    };
    const icon = document.createElement("span");
    icon.className = "text-blue-400 text-3xl mb-2 group-hover:-translate-x-1 transition-transform duration-200";
    icon.textContent = "←";
    const label = document.createElement("span");
    label.className = "text-blue-700 font-semibold";
    label.textContent = "Back";
    backBtn.append(icon, label);
    grid.appendChild(backBtn);
  }

  for (const child of entry.children) {
    if (search && !child.name.toLocaleLowerCase().normalize('NFKC').includes(search)) continue;

    const item = document.createElement("div");
    item.className =
      "flex flex-col items-center bg-gradient-to-br from-stone-100 via-stone-50 to-white rounded-2xl shadow-lg p-4 transition-transform duration-200 hover:scale-105 border border-stone-200 relative group";

    if (!child.is_dir) {
      const tooltip = document.createElement("div");
      tooltip.className =
        "absolute z-10 left-1/2 -translate-x-1/2 bottom-full mb-2 w-64 bg-gray-900 text-white text-xs rounded-lg shadow-lg px-4 py-2 opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-200";
      tooltip.style.whiteSpace = "pre-line";
      tooltip.innerText =
        `Name: ${child.name}\nSize: ${child.size ? formatSize(child.size) : "?"}\nModified: ${child.modified ? formatDate(child.modified) : "?"}`;
      item.appendChild(tooltip);
    }

    const icon = document.createElement("div");
    icon.className = "text-6xl mb-3 transition-transform group-hover:scale-110 text-blue-400 drop-shadow";
    icon.textContent = child.is_dir ? "📂" :
      child.file_type === "Audio" ? "🎵" :
      child.file_type === "Image" ? "🖼️" :
      child.file_type === "Video" ? "🎬" : "📄";

    const label = document.createElement("div");
    label.className = "text-center text-blue-900 font-semibold text-base truncate w-full px-2 mb-2";
    label.title = child.name;
    label.textContent = child.name;

    item.append(icon, label);

    const btnGroup = document.createElement("div");
    btnGroup.className = "flex flex-col items-center gap-2 mt-3";

    if (child.is_dir || child.is_browser_supported) {
      const openBtn = document.createElement("button");
      openBtn.textContent = "Open";
      openBtn.className = "px-4 py-1 bg-emerald-500 text-white rounded-full text-sm font-medium shadow hover:bg-emerald-600 transition-all duration-150";
      openBtn.onclick = (e) => {
        e.stopPropagation();
        child.is_dir ? setPath(child.path) : window.open(`/api/master/${encodeURIComponent(child.path)}`, "_blank");
      };
      btnGroup.appendChild(openBtn);
    }

    if (!child.is_dir) {
      const downloadBtn = document.createElement("a");
      downloadBtn.textContent = "Download";
      downloadBtn.href = `/api/master/${encodeURIComponent(child.path)}`;
      downloadBtn.download = child.name;
      downloadBtn.className = "px-4 py-1 bg-blue-500 text-white rounded-full text-sm font-medium shadow hover:bg-blue-600 transition-all duration-150 inline-block text-center";
      btnGroup.appendChild(downloadBtn);

      const updateBtn = document.createElement("button");
      updateBtn.textContent = "Update";
      updateBtn.className = "px-4 py-1 bg-amber-200 text-amber-900 rounded-full text-sm font-medium shadow hover:bg-amber-300 transition-all duration-150";

      const updateInput = document.createElement("input");
      updateInput.type = "file";
      updateInput.style.display = "none";
      updateInput.accept = "*/*";

      updateBtn.onclick = (e) => {
        e.stopPropagation();
        updateInput.click();
      };
      updateInput.onchange = async () => {
        if (!updateInput.files.length) return;
        const res = await updateFile(updateInput.files[0], child.path);
        alert(res.ok ? "File updated!" : "Update failed! " + await res.text());
        updateInput.value = "";
        refreshTree();
      };
      btnGroup.append(updateBtn, updateInput);

      const deleteBtn = document.createElement("button");
      deleteBtn.textContent = "Delete";
      deleteBtn.className = "px-4 py-1 bg-red-100 text-red-700 rounded-full text-sm font-medium shadow hover:bg-red-200 transition-all duration-150";
      deleteBtn.onclick = async (e) => {
        e.stopPropagation();
        if (!confirm(`Are you sure you want to delete '${child.name}'?`)) return;
        const res = await deleteFile(child.path);
        alert(res.ok ? "File deleted!" : "Delete failed! " + await res.text());
        refreshTree();
      };
      btnGroup.appendChild(deleteBtn);
    }

    item.appendChild(btnGroup);
    grid.appendChild(item);
  }

  return grid;
}
