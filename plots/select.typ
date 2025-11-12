#import "@preview/lilaq:0.3.0" as lq
#import "processing.typ": read_line_data
#import "drawing.typ": line_plot

#set page(width: auto, height: auto, margin: .5cm)

// Parse CSV input data
#let input = csv("../measurements/select.tsv", delimiter: "\t", row-type: dictionary)
#let (x_values, lines) = read_line_data(input)

#let slow_lines = (:)
#slow_lines.insert("Vers", lines.at("Vers"))
#for (name, line) in lines.pairs() {
    if calc.max(..line) > 20.0 {
        slow_lines.insert(name, lines.remove(name))
    }
}

#show: lq.set-diagram(width: 21cm, height: 14.8cm)
#set lq.mark(align: lq.marks.a3)

#for (title, dict) in (: "Select (Slowest)": slow_lines, "Select (Fastest)": lines).pairs() {
    line_plot(title, x_values, dict)
    pagebreak()
}