===description===
is_a()/is_string()/in_array()'s property-argument extraction now recognizes
a nullsafe (?->) property access the same as a plain (->) one, matching
every other comparison-narrowing arm in the file.
===config===
suppress=UnusedVariable,PossiblyNullArgument
===file===
<?php
class Item {}
class SpecialItem extends Item {}

class Box {
    public ?Item $item = null;
    public string|int|null $scalar = null;
}

function narrowsIsA(Box $b): void {
    if (is_a($b?->item, SpecialItem::class)) {
        /** @mir-check $b->item is SpecialItem */
        $_ = 1;
    }
}

function narrowsIsString(Box $b): void {
    if (is_string($b?->scalar)) {
        /** @mir-check $b->scalar is string */
        $_ = 1;
    }
}
===expect===
