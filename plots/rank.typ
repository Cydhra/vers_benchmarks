#import "@preview/lilaq:0.3.0" as lq
#import "processing.typ": read_line_data

#set page(width: auto, height: auto, margin: .5cm)

#let input = csv("../measurements/rank.tsv", delimiter: "\t", row-type: dictionary)
#let (x-val, lines) = read_line_data(input)

#let slow_lines = (:)
#slow_lines.insert("Vers", lines.at("Vers"))
#for (name, line) in lines.pairs() {
    if calc.max(..line) > 20.0 {
        slow_lines.insert(name, lines.remove(name))
    }
}

#show: lq.set-diagram(width: 21cm, height: 14.8cm)
#set lq.mark(align: lq.marks.a3)

#for (title, dict) in (: "Rank (Slowest)": slow_lines, "Rank (Fastest)": lines).pairs() {
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
                line,
                color: auto,
                label: name,
                stroke: (thickness: 2pt)
            ),)
        }
    )
    pagebreak()
}