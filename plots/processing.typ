/// Take an input dictionary that has the box-plot summary data, and convert it into a dictionary of
/// line plot y values for each measurement, and a single array of x values common to all measurements
#let read_line_data(input) = {
    let y_data = (:)
    let x_data = ()

    for arr in input {
        let runner = arr.at("Runner")
        if (runner not in y_data) {
            y_data.insert(runner, ())
        }
        y_data.at(runner).push(float(arr.at("Median")))

        if x_data.len() == 0 or x_data.last() != int(arr.at("Input Size")) {
            x_data.push(int(arr.at("Input Size")))
        }
    }

    (x_data, y_data)
}

