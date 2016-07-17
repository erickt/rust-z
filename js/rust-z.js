$(document).ready(function() {
    // Configure the page for a particular scene based on the URL
    moveToScene(parseUrl(document.URL));

    // When the `#` in the page URL changes, change the scene again!
    $(window).on('hashchange', function() {
	moveToScene(parseUrl(document.URL));
    });

    // And yup, navigate to #scene whene a scene button is clicked
    $("#section-controls > .scene").click(function() {
	if (this.id == "scene-overview") {
	    // The 'overview' button goes to the pretties URL.
	    // #overview would work as well, and not require a reload.
	    window.location = "./";
	} else {
	    window.location = "#" + this.innerHTML;
	}
    });

    // Show the bottom half of the battle
    $("#toggle-bottom").click(function() {
	$("#options").toggleClass("option-bottom");
    });
    $("#toggle-details").click(function() {
	$("#options").toggleClass("option-details");
    });
});

function parseUrl(url) {
    var scene = "overview";
    var filter = "";
    var select = "";
    if (url.indexOf("#") == -1) {
	// pass
    } else {
	var tail = url.substring(url.indexOf("#") + 1);
	var anchor = "";
	if (tail.indexOf("?") == -1) {
	    anchor = tail;
	} else {
	    anchor = tail.substring(0, tail.indexOf("?"));
	    var query = tail.substring(tail.indexOf("?") + 1);
	    var params = query.split("&");
	    for (param of params) {
		var parts = param.split("=");
		if (parts.length == 2) {
		    if (parts[0].startsWith("filter-")) {
			filter = parts[0] + "-" + parts[1];
		    }
		    if (parts[0].startsWith("select-")) {
			select = parts[0] + "-" + parts[1];
		    }
		}
	    }
	}

	if (anchor == "") {
	    // pass
	} else {
	    scene = anchor;
	}
    }

    return {
	scene: scene,
	filter: filter,
	select: select,
    };
}

function moveToScene(scene) {
    $("div#scene").removeClass();
    $("div#scene").addClass(scene.scene);
    $("div#filter").removeClass();
    $("div#filter").addClass(scene.filter);
    $("div#select").removeClass();
    $("div#select").addClass(scene.select);
    if (scene.select) {
	$("div#select").addClass("select-active");
    }
    $("div#section-controls .scene").removeClass("selected");
    $("div#section-controls .scene").
	filter(function(i, el) { return el.innerHTML == scene.scene; }).
	addClass("selected");
}
