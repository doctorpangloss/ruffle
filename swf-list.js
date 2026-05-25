const masterFrame = document.getElementById("master-frame");
const prFrame = document.getElementById("pr-frame");
const listRoot = document.getElementById("swf-list");
const filter = document.getElementById("swf-filter");
const customForm = document.getElementById("custom-swf-form");
const customUrl = document.getElementById("custom-swf-url");
const count = document.getElementById("swf-count");
const selectedTitle = document.getElementById("selected-title");
const selectedUrl = document.getElementById("selected-url");
const selectedIssue = document.getElementById("selected-issue");

let swfs = [];
let currentUrl = "";
let tlfCount = 0;

function shouldProxy(item) {
  return item.kind !== "included";
}

function proxiedUrl(url) {
  if (url.startsWith("https://swf-proxy.appmana.com/proxy/")) {
    return url;
  }
  return `https://swf-proxy.appmana.com/proxy/${url}`;
}

function normalized(item) {
  const tlf = item.tlfAdjacent ? " tlf textlayout swz" : "";
  return `${item.title} ${item.url} ${item.issue || ""} ${item.kind}${tlf}`.toLowerCase();
}

function load(item) {
  currentUrl = item.url;
  const loadUrl = shouldProxy(item) ? proxiedUrl(item.url) : item.url;
  const encoded = encodeURIComponent(loadUrl);
  masterFrame.src = `player-master.html?url=${encoded}`;
  prFrame.src = `player-pr.html?url=${encoded}`;
  selectedTitle.textContent = item.title;
  selectedUrl.textContent = item.url;
  selectedIssue.href = item.issue || "issues.html";
  selectedIssue.style.visibility = item.issue ? "visible" : "hidden";
  render();
}

function loadCustom(url) {
  load({
    title: "Custom SWF",
    url,
    kind: "custom",
    issue: "",
    tlfAdjacent: false,
  });
}

function render() {
  const query = filter.value.trim().toLowerCase();
  const visible = swfs.filter((item) => !query || normalized(item).includes(query));
  count.textContent = `${visible.length} of ${swfs.length} SWF entries, ${tlfCount} with adjacent TLF SWZ`;
  listRoot.replaceChildren(
    ...visible.slice(0, 500).map((item) => {
      const button = document.createElement("button");
      button.className = `swf-item${item.url === currentUrl ? " active" : ""}`;
      button.type = "button";
      const title = document.createElement("span");
      title.className = "swf-item-title";
      title.textContent = item.tlfAdjacent ? `TLF ${item.title}` : item.title;
      const url = document.createElement("span");
      url.className = "swf-item-url";
      url.textContent = item.url;
      button.append(title, url);
      button.addEventListener("click", () => load(item));
      return button;
    }),
  );
}

Promise.all([
  fetch("data/textlayout-examples.json").then((response) => response.json()),
])
  .then(([data]) => {
    swfs = data.items.map((item) => ({
      ...item,
      tlfAdjacent: true,
      adjacentSwz: item.adjacentSwz || [],
    }));
    tlfCount = swfs.length;
    filter.addEventListener("input", render);
    customForm.addEventListener("submit", (event) => {
      event.preventDefault();
      const url = customUrl.value.trim();
      if (url) {
        loadCustom(url);
      }
    });
    const initialUrl = new URLSearchParams(window.location.search).get("url");
    if (initialUrl) {
      customUrl.value = initialUrl;
      loadCustom(initialUrl);
    } else {
      load(swfs[0]);
    }
  })
  .catch((error) => {
    count.textContent = `Failed to load SWF list: ${error}`;
  });
