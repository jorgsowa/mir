===description===
A bare subclass with no receiver-supplied type args at all (`new IntBox()`,
not `@var IntBox<int>`) still resolves an inherited `@var T` property
through a `@extends Box<int>`-fixed ancestor template — the own-type-args
substitution path early-returned the raw property type before reaching
`inherited_template_bindings` whenever the receiver itself had zero args.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
}

/** @extends Box<int> */
class IntBox extends Box {}

function test(): void {
    $box = new IntBox();
    /** @mir-check $box->value is int */
    $_ = $box->value;
}

class PlainBox extends Box {}

function noFixedTemplateStaysRaw(): void {
    // No `@extends` type args to resolve — the property legitimately
    // stays the raw, unsubstituted template.
    $box = new PlainBox();
    /** @mir-check $box->value is T */
    $_ = $box->value;
}
===expect===
