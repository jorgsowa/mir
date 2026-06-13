===description===
Strict null comparisons narrow types correctly
===config===
suppress=MissingReturnType
===file===
<?php
function testTripleEqNull(int|null $x) {
    if ($x === null) {
        return null;
    }
    return $x + 1;
}

function testTripleNotEqNull(int|null $x) {
    if ($x !== null) {
        return $x + 1;
    }
    return null;
}

function testNullOnLeft(string|null $x) {
    if (null === $x) {
        return null;
    }
    strlen($x);
}

function testNullNotEqOnLeft(string|null $x) {
    if (null !== $x) {
        strlen($x);
    }
}
===expect===
