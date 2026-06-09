#import "@preview/lilaq:0.3.0" as lq
#import "processing.typ": read_line_data
#import "drawing.typ": line_plot

#set page(width: auto, height: auto, margin: .5cm)

#let input = csv("../measurements/elias_fano_access.tsv", delimiter: "\t", row-type: dictionary)
#let (x_values, lines) = read_line_data(input)

#show: lq.set-diagram(width: 21cm, height: 14.8cm)
#set lq.mark(align: lq.marks.a3)

#let ef_compare = (:)
#ef_compare.insert("Vers", lines.at("Vers"))
#ef_compare.insert("elias-fano", lines.remove("eliasfano"))

#line_plot("Elias Fano Random Access", x_values, ef_compare, ylim: 1000)
#pagebreak()
#line_plot("Elias Fano Random Access", x_values, lines)
#pagebreak()

#let input = csv("../measurements/elias_fano_successor.tsv", delimiter: "\t", row-type: dictionary)
#let (x_values, lines) = read_line_data(input)

#line_plot("Elias Fano Successor", x_values, lines)