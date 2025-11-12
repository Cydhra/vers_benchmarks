/// Take an input dictionary that has
#let read_line_data(input) = {
    let lines = (:)
    let x-val = ()

    for arr in input {
        let runner = arr.at("Runner")
        if (runner not in lines) {
            lines.insert(runner, ())
        }
        lines.at(runner).push(float(arr.at("Median")))

        if x-val.len() == 0 or x-val.last() != int(arr.at("Input Size")) {
            x-val.push(int(arr.at("Input Size")))
        }
    }

    (x-val, lines)
}

