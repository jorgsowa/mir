===description===
`$obj->prop`'s own inferred type must include null when `$obj` is nullable
— PHP 8 evaluates a `->` access on a null receiver to null (a warning, not
fatal). analyze_property_access previously never widened for this,
missing real bugs like returning `$obj->prop` from a `string`-declared
function when `$obj` could be null.
===config===
suppress=MissingReturnType
===file===
<?php
class Foo {
    public string $bar = 'x';
}

// Positive: real bug now caught.
function returnsPossiblyNull(?Foo $obj): string {
    return $obj->bar;
}

// Negative: non-nullable receiver, no over-widening.
function nonNullableReceiver(Foo $obj): string {
    return $obj->bar;
}

// Negative: receiver narrowed to non-null before the access.
function narrowedFirst(?Foo $obj): string {
    if ($obj === null) {
        return "";
    }
    return $obj->bar;
}
===expect===
NullableReturnStatement@8:4-8:21: Return type 'string|null' is not compatible with declared 'string'
PossiblyNullPropertyFetch@8:11-8:20: Cannot access property $bar on possibly null value
