===description===
Parenthesized guards are handled correctly
===config===
suppress=MissingParamType,MissingReturnType,MixedArgument
===file===
<?php
function testParenthesized($x) {
    if ((is_string($x))) {
        strlen($x);
    }
}

function testParenthesizedNegated(string|null $x) {
    if ((!is_null($x))) {
        strlen($x);
    }
}

function testDoubleParenthesized($x) {
    if (((is_int($x)))) {
        return $x + 1;
    }
}

function testParenthesizedComparison(int|null $x) {
    if (($x === null)) {
        return null;
    }
    return $x;
}
===expect===
