===description===
@psalm-extends is recognized as an alias for @extends (the vendor prefix
was previously only accepted for @phpstan-extends, not @psalm-extends).
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
}

/** @psalm-extends Box<int> */
class IntBox extends Box {}

function test(): void {
    $box = new IntBox();
    /** @mir-check $box->value is int */
    $_ = $box->value;
}
===expect===
