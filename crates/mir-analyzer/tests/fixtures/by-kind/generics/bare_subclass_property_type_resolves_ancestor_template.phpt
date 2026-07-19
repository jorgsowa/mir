===description===
A bare subclass that doesn't redeclare @template (`class IntBox extends Box
{}`) still resolves an inherited `@var T` property through the ancestor's
template the same way a directly-generic class already does.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
}

class IntBox extends Box {}

function test(): void {
    /** @var IntBox<int> $box */
    $box = new IntBox();
    /** @mir-check $box->value is int */
    $_ = $box->value;
}
===expect===
