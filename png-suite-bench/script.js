const results = await(await fetch('./test_results.json')).json();

results.processed_images.sort();
results.processed_images.forEach(add_test_comparison);

function add_test_comparison(test_name) {
    const comparison = document.createElement("div");
    comparison.classList.add("comparison");
    comparison.id = test_name;
    const title = document.createElement("p");
    title.append(test_name);
    comparison.appendChild(title);
    const orig = document.createElement("img");
    orig.src = `./images/${test_name}-orig.png`;
    orig.classList.add("orig");
    comparison.appendChild(orig);
    const spng = document.createElement("img");
    spng.src = `./images/${test_name}-spng.png`;
    spng.classList.add("spng");
    comparison.appendChild(spng);
    document.querySelector(".results").appendChild(comparison);
}

