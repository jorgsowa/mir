===description===
FN: a single, non-union generic param `Bar<T>` suppressed ALL argument
checking whenever the class itself was fully known but its own type args
mentioned an unresolved template — not just when the same case occurs
inside a union (see known_limitation_template_union_member_suppresses_
whole_union.phpt). `param_contains_template_or_unknown` treated any nested
template as "forgive the whole param," even when the argument's own class
(or complete absence of a class, for a bare scalar) could never satisfy
`Bar` regardless of what `T` resolves to.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
class Bar {
    /** @param T $x */
    public function __construct($x) {}
}

class Unrelated {}

/**
 * @template T
 * @param Bar<T> $x
 */
function takesBar($x): void {}

takesBar("plain-string");
takesBar(new Unrelated());
===expect===
InvalidArgument@16:9-16:23: Argument $x of takesBar() expects 'Bar<T>', got '"plain-string"'
InvalidArgument@17:9-17:24: Argument $x of takesBar() expects 'Bar<T>', got 'Unrelated'
