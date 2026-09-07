===description===
`!empty($base['a']['b'])` proves both `$base['a']` present/non-null and
`$base['a']['b']` present/truthy, but narrow_empty_shape_key only ever
matched a single `ArrayAccess` node whose array was a bare variable — a
nested `!empty()` bailed via that check as soon as the array was itself an
ArrayAccess, so it narrowed nothing at any level, unlike its `isset()`
sibling.
===config===
suppress=MixedAssignment
===file===
<?php
/** @param array{a: array{b: ?string}} $x */
function f(array $x): string {
    if (!empty($x['a']['b'])) {
        $b = $x['a']['b'];
        /** @mir-check $b is non-empty-string */
        return $b;
    }
    return '';
}

/** @param array{a?: array{b?: array{c?: string}}} $data */
function threeLevels(array $data): string {
    if (!empty($data['a']['b']['c'])) {
        $c = $data['a']['b']['c'];
        /** @mir-check $c is non-empty-string */
        return $c;
    }
    return "none";
}
===expect===
