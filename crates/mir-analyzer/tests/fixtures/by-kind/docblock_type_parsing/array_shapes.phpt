===description===
`array{...}`/`list{...}` shapes: keyed entries, `?` optional markers,
positional (unkeyed) entries auto-indexed from 0, quoted non-identifier
keys, and nesting. `object{...}` has no dedicated shape atom and is
approximated as plain `object` (property shape is lost — see
parser/docblock/types.rs).
===config===
suppress=MissingReturnType,MissingParamType,ForbiddenCode
===file===
<?php
function check_array_shape_basic($x) {
    /**
     * @var array{a: int, b: string} $x
     * @mir-check $x is array{a: int, b: string}
     */
    var_dump($x);
}

function check_array_shape_optional($x) {
    /**
     * @var array{a: int, b?: string} $x
     * @mir-check $x is array{a: int, b?: string}
     */
    var_dump($x);
}

function check_array_shape_positional($x) {
    /**
     * @var array{int, string} $x
     * @mir-check $x is array{0: int, 1: string}
     */
    var_dump($x);
}

function check_list_shape($x) {
    /**
     * @var list{int, string} $x
     * @mir-check $x is array{0: int, 1: string}
     */
    var_dump($x);
}

function check_array_shape_quoted_key($x) {
    /**
     * @var array{'foo-bar': int} $x
     * @mir-check $x is array{'foo-bar': int}
     */
    var_dump($x);
}

function check_array_shape_nested($x) {
    /**
     * @var array{a: array{b: int}} $x
     * @mir-check $x is array{a: array{b: int}}
     */
    var_dump($x);
}

function check_array_shape_generic_value($x) {
    /**
     * @var array{a: array<int, string>} $x
     * @mir-check $x is array{a: array<int, string>}
     */
    var_dump($x);
}

function check_object_shape($x) {
    /**
     * @var object{a: int} $x
     * @mir-check $x is object
     */
    var_dump($x);
}
===expect===
