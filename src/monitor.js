document.addEventListener("DOMContentLoaded", function(event) {
    const socket = new WebSocket("ws://{{WSHOST}}:9001");

    socket.onopen = (event) => {
        console.log("Connected");
    };

    let last_update = -1;
    setLastUpdate(last_update);

    let total_records = document.getElementById("total-records");
    let failed_records = document.getElementById("failed-records");

    let last_record = null;
    init_plot();
    socket.onmessage = async (event) => {
        let msg;
        total_records.innerText = parseInt(total_records.innerText) + 1;
        try {
            msg = JSON.parse(event.data);
            last_record = msg;
        } catch (e) {
            console.log("Can't parse JSON: " + e);
            console.log(event.data);
            failed_records.innerText = parseInt(failed_records.innerText) + 1;
            return;
        }

        last_update = 0;
        setLastUpdate(last_update);
        extend_plot(msg);

        // Runtime
        let secs = msg.run_time.secs;
        document.getElementById("runtime").innerText = Math.floor(secs / 3600) + "h " + Math.floor(secs / 60) % 60 + "m " + secs % 60 + "s";
        document.getElementById("executions").innerText = new Intl.NumberFormat().format(msg.executions);
        document.getElementById("objectives").innerText = new Intl.NumberFormat().format(msg.objectives);
        document.getElementById("corpus").innerText = new Intl.NumberFormat().format(msg.corpus);
        document.getElementById("executions").innerText = new Intl.NumberFormat().format(msg.executions);
        document.getElementById("exec_sec").innerText = new Intl.NumberFormat().format(msg.exec_sec);

        let table = document.getElementById("clients");

        for (let i = 0; i < msg.clients.length; i++) {
            if (table.rows.length <= i) {
                let row = table.insertRow();
                for (let j = 0; j < 5; j++) {
                    row.insertCell(-1);
                }
                row.onclick = function () {
                    document.getElementById("client-overview-id").innerText = i;
                    let list = document.getElementById("client-overview-user-stats");
                    let k = 0;
                    for (const key in last_record.clients[i].user_monitor) {
                        let plot_link = document.createElement("a");
                        plot_link.setAttribute("href", "#plot-client" + i + "-userdata-" + key);
                        plot_link.innerText = "📈";
                        let li = document.createElement("li");
                        li.appendChild(document.createTextNode(key + ": " + format_user_stat(last_record.clients[i].user_monitor[key]) + " "));
                        li.appendChild(plot_link);

                        if (list.childNodes.length > k) {
                            list.replaceChild(li, list.childNodes[k]);
                        } else {
                            list.appendChild(li);
                        }
                        k++;
                    }

                    console.log(last_record.clients[i].user_monitor);
                    for (let k = list.children.length - 1; k >= Object.keys(last_record.clients[i].user_monitor).length ; k--) {
                        list.childNodes[k].remove();
                    }

                    document.getElementById("client-overview").style = "";
                };
            }
            let row = table.rows[i];
            let data = msg.clients[i];

            row.cells[0].innerText = "Client #" + i;
            row.cells[1].innerText = new Intl.NumberFormat().format(data.corpus_size);
            row.cells[1].appendChild(get_plotlink(i, "corpus_size"));
            row.cells[2].innerText = new Intl.NumberFormat().format(data.objective_size);
            row.cells[2].appendChild(get_plotlink(i, "objective_size"));
            row.cells[3].innerText = new Intl.NumberFormat().format(data.executions);

        }
    };

    setInterval(function () {
        if (last_update < 0) {
            return;
        }
        last_update++;
        setLastUpdate(last_update);
    }, 1000);
});

window.addEventListener("hashchange", function () {
   location.reload();
});

function get_plotlink(id, target) {
    let plot_link = document.createElement("a");
    plot_link.setAttribute("href", "#plot-client" + id + "-" + target);
    plot_link.innerText = "📈";
    return plot_link;
}

function setLastUpdate(s) {
    let last_update_elem = document.getElementById("last-update");
    if (s < 0) {
        last_update_elem.innerText = "never";
    } else {
        last_update_elem.innerText = new Intl.RelativeTimeFormat().format(-s, 'second');
    }
}

function format_user_stat(stat) {
    let type = Object.keys(stat)[0];
    if (type === "Number") {
        return stat[type];
    } else if (type === "Ratio") {
        return stat[type][0] + "/" + stat[type][1];
    }

    console.log(Object.keys(stat));
    console.log(stat);
}

function get_user_value(stat) {
    if (stat === undefined) {
        return 0;
    }

    let type = Object.keys(stat)[0];
    if (type === "Number") {
        return stat[type];
    } else if (type === "Ratio") {
        return parseInt(stat[type][0]) / parseInt(stat[type][1]) * 100;
    }

    console.log(Object.keys(stat));
    console.log(stat);
}

function extend_plot(data) {
    let y_data;
    let which = window.location.hash;

    if (which.startsWith("#plot-client")) {
        const re = new RegExp('#plot-client([0-9]*)-([a-z_]*)-?(.*)?');
        let match = which.match(re);

        let client_data = data.clients[parseInt(match[1])];
        console.log(client_data);
        if (match[2] === "userdata" && match[3] !== undefined) {
            y_data = get_user_value(client_data.user_monitor[match[3]]);
        } else {
            y_data = client_data[match[2]];
        }

    } else if (which === "#plot-objectives") {
        y_data = data.objectives;
    } else if (which === "#plot-corpus") {
        y_data = data.corpus;
    } else {
        y_data = data.exec_sec;
    }

    Plotly.extendTraces("executions-plot", {
        x: [[data.run_time.secs]],
        y: [[y_data]]
    }, [0]);
}

function init_plot() {
    let name;
    let which = window.location.hash;

    if (which.startsWith("#plot-client")) {
        const re = new RegExp('#plot-client([0-9]*)-([a-z_]*)-?(.*)?');
        let match = which.match(re);

        name = "Client #" + match[1];
        if (match[3] === undefined) {
            name += " " + match[2];
        } else {
            name += " " + match[3];
        }
    } else if (which === "#plot-corpus") {
        name = "corpus entries";
    } else if (which === "#plot-objectives") {
        name = "objectives entries";
    } else {
        name = "exec/s";
    }

    Plotly.newPlot("executions-plot", [{
        x: [],
        y: [],
        mode: 'scatter',
        name: name,
        showlegend: true
    }], {margin: {t: 0},  yaxis: {title: { text: name}, rangemode: 'tozero'}, xaxis: {title: { text: 'runtime'}, type: "linear", tickformat: "~s"}}, {displayModeBar: false});
}