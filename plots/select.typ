#import "@preview/lilaq:0.3.0" as lq

#set page(width: auto, height: auto, margin: .5cm)

// Parse CSV input data
#let input = csv("../measurements/select.tsv", delimiter: "\t", row-type: dictionary)
#let lines = (:)
#let x-val = ()

#for arr in input {
    let runner = arr.at("Runner")
    if (runner not in lines) {
        lines.insert(runner, ((), ()))
    }
    lines.at(runner).at(0).push(float(arr.at("Median")))
    lines.at(runner).at(1).push((: "m": float(arr.at("Q1")), "p": float(arr.at("Q3"))))
    
    if x-val.len() == 0 or x-val.last() != int(arr.at("Input Size")) {
        x-val.push(int(arr.at("Input Size")))
    }
}

#let slow_lines = (:)
#slow_lines.insert("Vers", lines.at("Vers"))
#for (name, line) in lines.pairs() {
    if calc.max(..line.at(0)) > 20.0 {
        slow_lines.insert(name, lines.remove(name))
    }
}

#show: lq.set-diagram(width: 21cm, height: 14.8cm)
#set lq.mark(align: lq.marks.a3)

#for (title, dict) in (: "Select (Slowest)": slow_lines, "Select (Fastest)": lines).pairs() {
    lq.diagram(
        title: title,
        xlabel: "Elements",
        ylabel: "Time (ns)",
        xscale: "log",
        cycle: lq.color.map.petroff8,
        ylim: (0, auto),

        ..for (name, line) in dict.pairs() {
            (lq.plot(
                x-val,
                line.at(0),
                color: auto,
                label: name,
                stroke: (thickness: 2pt)
            ),)
        }
    )
    pagebreak()
}