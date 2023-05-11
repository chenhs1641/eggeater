mod infra;

// Your tests go here!
success_tests! {
    {
        name: basic,
        file: "basic.snek",
        expected: "4",
    },
    {
        name: no_use_func,
        file: "no_use_func.snek",
        expected: "4",
    },
    {
        name: fact,
        file: "fact.snek",
        input: "10",
        expected: "3628800",
    },
    {
        name: even_odd_1,
        file: "even_odd.snek",
        input: "10",
        expected: "10\ntrue\ntrue",
    },
    {
        name: even_odd_2,
        file: "even_odd.snek",
        input: "9",
        expected: "9\nfalse\nfalse",
    }
}

runtime_error_tests! {
    {
        name: type_error,
        file: "type_error.snek",
        expected: "invalid argument",
    }
}

static_error_tests! {
    {
        name: duplicate_params,
        file: "duplicate_params.snek",
        expected: "",
    }
}
