window.jQuery = false;
hljs.initHighlightingOnLoad();

document.getElementById("mkdocs-search-query").addEventListener("keyup", function () {
    if (this.value === "") {
        document.querySelector("[role=main]").classList.remove("searching");
    } else {
        document.querySelector("[role=main]").classList.add("searching");
    }
});

document.getElementById("sidebar-toggle").addEventListener("click", function () {
    document.querySelector("[role=navigation]").classList.toggle("visible");
});

document.querySelector("[role=main]").addEventListener("click", function () {
    document.querySelector("[role=navigation]").classList.remove("visible");
});
