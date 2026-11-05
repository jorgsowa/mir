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
    /** @mir-check $properties is array<string, mixed> */
    echo count($properties);
    /** @mir-check $publicProperties is array<string, mixed> */
    echo count($publicProperties);
    /** @mir-check $closed is mixed */
    echo gettype($closed);
}

f(1, ['x' => 1], ['x' => 1], null);
