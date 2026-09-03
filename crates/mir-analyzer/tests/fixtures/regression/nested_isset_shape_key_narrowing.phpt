===description===
`isset($base['a']['b'])` proves BOTH `$base['a']` and `$base['a']['b']`
present and non-null, but narrow_isset_shape_key only ever handled a
single-level access — it bailed via extract_var_name as soon as the array
base was itself an ArrayAccess, so a nested isset() narrowed nothing at any
level. Also covers the companion bug this exposed: PossiblyNullArrayAccess
fired even on the isset() condition's OWN expression, because unlike its
sibling PossiblyInvalidArrayAccess check it never respected
in_existence_check — isset() never triggers PHP runtime warnings for its
own argument chain.
===config===
suppress=MixedAssignment
===file===
<?php
/** @param array{address?: array{city?: string}} $data */
function cityOf(array $data): string {
    if (isset($data['address']['city'])) {
        $city = $data['address']['city'];
        /** @mir-check $city is string */
        return $city;
    }
    return "no city";
}

/** @param array{a?: array{b?: array{c?: string}}} $data */
function threeLevels(array $data): string {
    if (isset($data['a']['b']['c'])) {
        $c = $data['a']['b']['c'];
        /** @mir-check $c is string */
        return $c;
    }
    return "none";
}
===expect===
