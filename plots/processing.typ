#let read_line_data(input) = {
    let lines = (:)
    let x-val = ()

    for arr in input {
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

    (x-val, lines)
}

