#import "@preview/lilaq:0.3.0" as lq

/// Draw a plot with multiple lines for each measurement in a benchmark
#let line_plot(title, x_data, y_data, ylim: auto) = lq.diagram(
    title: title,
    xlabel: "Elements",
    ylabel: "Time (ns)",
    xscale: "log",
    cycle: lq.color.map.petroff8,
    ylim: (0, ylim),
    
    ..for (name, line) in y_data.pairs() {
         (lq.plot(
             x_data,
             line,
             color: auto,
             label: name,
             stroke: (thickness: 2pt)
         ),)
    }
)