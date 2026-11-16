===description===
a zero float literal does not fit non-empty-scalar; other floats still do
===file===
<?php
/** @param non-empty-scalar $v */
function acceptsNonEmptyScalar($v): void {
    echo gettype($v);
}

/** @param float $v */
function acceptsFloat($v): void {
    echo $v;
}

function caller(float $f): void {
    acceptsNonEmptyScalar(0.0);
//                        ^^^ InvalidArgument: Argument $v of acceptsNonEmptyScalar() expects 'non-empty-scalar', got '0.0'
    acceptsNonEmptyScalar(-0.0);
//                        ^^^^ InvalidArgument: Argument $v of acceptsNonEmptyScalar() expects 'non-empty-scalar', got '0.0'
    acceptsNonEmptyScalar(1.5);
    acceptsNonEmptyScalar(-2.5);
    acceptsNonEmptyScalar($f);
    acceptsNonEmptyScalar('a');
    acceptsNonEmptyScalar(true);
    acceptsFloat(0.0);
    acceptsFloat(-0.0);
}
