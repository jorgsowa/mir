===file===
<?php
namespace App;

class Foo { public int $x = 0; }

/**
 * @param __benevolent<int|false> $benevolent
 * @param properties-of<Foo> $properties
 * @param public-properties-of<Foo> $publicProperties
 * @param closed-resource $closed
 */
function f($benevolent, $properties, $publicProperties, $closed): void {
    /** @mir-check $benevolent is int|false */
    echo $benevolent;
    /** @mir-check $properties is array{'x': int} */
    echo count($properties);
    /** @mir-check $publicProperties is array{'x': int} */
    echo count($publicProperties);
    /** @mir-check $closed is closed-resource */
    echo gettype($closed);
}

f(1, ['x' => 1], ['x' => 1], null);
//                           ^^^^ NullArgument: Argument $closed of f() cannot be null
