#import "@preview/lilaq:0.3.0" as lq

#set page(width: auto, height: auto, margin: .5cm)

// Parse CSV input data
#let input = csv("../measurements/rank.csv")
#let lines = (:)
#let x-val = ()

#for arr in input [
    #if (arr.at(1) not in lines) {
        lines.insert(arr.at(1), ((), ()))
    }
    #(lines.at(arr.at(1)).at(0).push(float(arr.at(3))))
    #(lines.at(arr.at(1)).at(1).push((: "m": float(arr.at(4)), "p": float(arr.at(5)))))

    #if x-val.len() == 0 or x-val.last() != int(arr.at(2)) {
        x-val.push(int(arr.at(2)))
    }
]
#let slow_lines = (:)
#slow_lines.insert("Vers", lines.at("Vers"))
#for (name, line) in lines.pairs() {
    if calc.max(..line.at(0)) > 5.0 {
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
    linebreak()
}