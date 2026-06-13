===description===
Negated type guards narrow correctly
===config===
suppress=MissingReturnType
===file===
<?php
function testNotNull(string|null $x) {
    if (!is_null($x)) {
        strlen($x);
    }
}

function testNotString(int|string $x) {
    if (!is_string($x)) {
        return $x + 1;
    }
}

function testNotInt(int|string $x) {
    if (!is_int($x)) {
        strlen($x);
    }
}

function testNotArray(array|string $x) {
    if (!is_array($x)) {
        strlen($x);
    }
}
===expect===
