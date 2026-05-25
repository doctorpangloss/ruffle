const summary = document.getElementById("summary");
const filter = document.getElementById("filter");
const issuesRoot = document.getElementById("issues");

function labelNames(issue) {
  return (issue.labels || []).map((label) => label.name).filter(Boolean);
}

function matches(issue, query) {
  if (!query) {
    return true;
  }
  const text = [
    issue.number,
    issue.title,
    issue.state,
    ...labelNames(issue),
    ...(issue.matched_terms || []),
  ]
    .join(" ")
    .toLowerCase();
  return text.includes(query);
}

function render(data) {
  const query = filter.value.trim().toLowerCase();
  const visible = data.issues.filter((issue) => matches(issue, query));
  summary.textContent = `${visible.length} of ${data.count} candidate upstream text issues shown. Search terms: ${data.terms.join(", ")}.`;
  issuesRoot.replaceChildren(
    ...visible.map((issue) => {
      const row = document.createElement("article");
      row.className = "issue";
      const labels = labelNames(issue)
        .map((name) => `<span class="label">${name}</span>`)
        .join("");
      row.innerHTML = `
        <div class="issue-title"><a href="${issue.url}">#${issue.number}: ${issue.title}</a></div>
        <div class="issue-details">${issue.state} | updated ${issue.updatedAt} | matched ${(issue.matched_terms || []).join(", ")}</div>
        <div>${labels}</div>
      `;
      return row;
    }),
  );
}

fetch("data/text-issues.json")
  .then((response) => response.json())
  .then((data) => {
    filter.addEventListener("input", () => render(data));
    render(data);
  })
  .catch((error) => {
    summary.textContent = `Failed to load issue inventory: ${error}`;
  });
