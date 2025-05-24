export async function fetchMasterTree() {
  const res = await fetch("/api/master.json");
  return res.json();
}

export async function uploadFile(formData, currentPath) {
  formData.append("target_path", currentPath);
  const res = await fetch("/api/upload", {
    method: "POST",
    body: formData,
  });
  return res;
}

export async function updateFile(file, replacePath) {
  const formData = new FormData();
  formData.append("file", file);
  formData.append("replace_path", replacePath);
  const res = await fetch("/api/update", {
    method: "POST",
    body: formData,
  });
  return res;
}

export async function deleteFile(path) {
  const res = await fetch("/api/delete", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path })
  });
  return res;
}

export async function createFolder(path) {
  const res = await fetch("/api/create_folder", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ path })
  });
  return res;
}
