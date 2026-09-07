===description===
`strpos($obj->prop, 'needle') !== false` / `array_search($obj->prop,
$haystack, true) !== false` must also prove `$obj` itself is non-null —
same reasoning as the already-fixed `in_array()` sibling: a found result
proves the property read wasn't null-derived. The not-found direction
(last two functions) proves nothing about the receiver.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Box {
    public ?string $tag = null;
    public function ping(): void {}
}

function viaStrpos(?Box $b): void {
    if (strpos($b->tag, 'needle') !== false) {
        $b->ping();
    }
}

function viaArraySearch(?Box $b): void {
    if (array_search($b->tag, ['a', 'b', 'c'], true) !== false) {
        $b->ping();
    }
}

// Negative: not-found proves nothing about $b itself.
function viaStrposNotFound(?Box $b): void {
    if (strpos($b->tag, 'needle') === false) {
        $b->ping();
    }
}

function viaArraySearchNotFound(?Box $b): void {
    if (array_search($b->tag, ['a', 'b', 'c'], true) === false) {
        $b->ping();
    }
}
===expect===
PossiblyNullArgument@8:15-8:22: Argument $haystack of strpos() might be null
PossiblyNullPropertyFetch@8:15-8:22: Cannot access property $tag on possibly null value
PossiblyNullPropertyFetch@14:21-14:28: Cannot access property $tag on possibly null value
PossiblyNullArgument@21:15-21:22: Argument $haystack of strpos() might be null
PossiblyNullPropertyFetch@21:15-21:22: Cannot access property $tag on possibly null value
PossiblyNullMethodCall@22:8-22:18: Cannot call method ping() on possibly null value
PossiblyNullPropertyFetch@27:21-27:28: Cannot access property $tag on possibly null value
PossiblyNullMethodCall@28:8-28:18: Cannot call method ping() on possibly null value
