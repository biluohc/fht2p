TABLE = "table";
SORTS = ["N0", "N1", "D0", "D1", "S0", "S1"];
SORT = "";
SPACEHOLDER = "&nbsp;";

h1_human(8, 16);
date_human(1);
size_human(2);

function spaceholder(num) {
    var str = "";
    for (var i = 0; i < num; i++) {
        str += SPACEHOLDER;
    }
    return str;
}

function h1_human(num0, num1) {
    var h1 = document.getElementsByTagName("h1")[0];
    var content = h1.innerHTML;
    content = spaceholder(num0) + content;
    h1.innerHTML = content;

    var client = document.getElementById("client");
    var content = client.innerHTML;
    content = spaceholder(num1) + content;
    client.innerHTML = content;
}

function sort_by(cell) {
    sorter(cell);
    date_human(1);
    size_human(2);
}

function sorter(cell) {
    var table = document.getElementById(TABLE);
    var tablebody = table.tBodies[0];
    var rows = new Array;
    var rows_head = new Array;
    var idx = 1; //t_head
    if (table.rows[1].cells[0].innerText.toString() === "../ Parent Directory") {
        idx += 1;
    }
    for (var i = 0; i < table.rows.length; i++) {
        if (i < idx) {
            rows_head.push(table.rows[i]);
        } else {
            rows.push(table.rows[i]);
        }
    }

    var sort = SORT;
    console.log("cell/sort: " + cell + "/" + sort);
    switch (cell) {
        case 0:
            if (sort === SORTS[0]) {
                rows.reverse();
                sort = SORTS[1];
            } else {
                rows.sort(comparer(cell, "str"));
                sort = SORTS[0];
            }
            break;
        case 1:
            if (sort === SORTS[2]) {
                rows.reverse();
                sort = SORTS[3];
            } else {
                rows.sort(comparer_data(cell, "date"));
                sort = SORTS[2];
            }
            break;
        case 2:
            if (sort === SORTS[4]) {
                rows.reverse();
                sort = SORTS[5];
            } else {
                rows.sort(comparer_data(cell, "size"));
                sort = SORTS[4];
            }
            break;
        default:
    }
    SORT = sort;

    var rows_head = rows_head.concat(rows);
    // for (var i = 0; i < table.rows.length; i++) {
    //     console.log(i + ": " + table.rows[i].cells[cell].innerText + " -->" + rows_head[i].cells[cell].innerText);
    // }
    var tbodyFragment = document.createDocumentFragment();
    for (var i = 0; i < rows_head.length; i++) {
        tbodyFragment.appendChild(rows_head[i]);
    }
    tablebody.appendChild(tbodyFragment);
}

function str_to_type(value, type) {
    switch (type) {
        case "size":
            return parseInt(value);
        case "date":
            return new Date(Date.parse(value));
        default:
            return value;
    }
}

function comparer(cell, type) {
    return function(row0, row1) {
        var cell0 = str_to_type(row0.cells[cell].innerText, type);
        var cell1 = str_to_type(row1.cells[cell].innerText, type);
        if (cell0 < cell1) {
            return -1;
        } else if (cell0 > cell1) {
            return 1;
        } else {
            return 0;
        }
    };
}

function comparer_data(cell, type) {
    return function(row0, row1) {
        var cell0 = str_to_type(row0.cells[cell].getAttribute("data"), type);
        var cell1 = str_to_type(row1.cells[cell].getAttribute("data"), type);
        // console.log("cell0/cell1: " + cell0 + "/" + cell1);
        if (cell0 < cell1) {
            return -1;
        } else if (cell0 > cell1) {
            return 1;
        } else {
            return 0;
        }
    };
}

function size_human(cell) {
    var table = document.getElementById(TABLE);
    for (var i = 1; i < table.rows.length; i++) {
        var td = table.rows[i].cells[cell];
        // alert(parseInt(td.getAttribute("data")));
        var data = td.getAttribute("data");
        var size = parseInt(data);
        var size = size_humaner(size);
        if (size.match("NaN")) {
            continue;
        }
        td.innerHTML = spaceholder(5) + size;
    }
}

function size_humaner(size) {
    // B，KB，MB，GB，TB，PB，EB，ZB，YB，BB
    var units = ["", "K", "M", "G", "T", "P", "E", "Z", "Y", "B"];
    var s = size;
    var count = 0;
    while (s / 1024 > 1) {
        s = s / 1024;
        count += 1;
    }
    return s.toFixed(2) + units[count];
}

function date_human(cell) {
    var table = document.getElementById(TABLE);
    for (var i = 1; i < table.rows.length; i++) {
        var td = table.rows[i].cells[cell];
        // alert(td.getAttribute("data"));
        var data = td.getAttribute("data");
        var date = new Date(data);
        var date = date_humaner(date);
        if (date.match("NaN")) {
            continue;
        }
        td.innerHTML = spaceholder(10) + date;
    }
}

function date_humaner(date) {
    // 2017-0226 00:42:48
    return date.getFullYear() + "-" + fixblanks(date.getMonth() + 1) + fixblanks(date.getDate()) + " " + fixblanks(date.getHours()) + ":" + fixblanks(date.getMinutes()) + ":" + fixblanks(date.getSeconds());

    function fixblanks(num) {
        if (num < 10) {
            return "0" + num;
        } else {
            return num;
        }
    }
}