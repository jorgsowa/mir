===description===
Negative control for the promoted-property docblock-literal-union fix: a
genuinely missing arm still flags, naming the uncovered literal.
===config===
suppress=UnusedParam
===file===
<?php
final class Attr {
    /** @param 'a'|'b' $value */
    public function __construct(public string $value) {}
}
function f(Attr $attr): int {
    return match ($attr->value) { 'a' => 1 };
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnhandledMatchCondition: Unhandled match condition: "b"
}
===expect===
