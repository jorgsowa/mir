===description===
D4: an arrow function's `=> expr` is exactly one implicit `return expr;` — a
nullable property read against a non-nullable declared return type must flag
the same way an equivalent `function(){...}` closure already does.
===config===
suppress=UnusedVariable,MissingClosureReturnType,MissingPropertyType
===file===
<?php
class Holder {
    /** @var null|string */
    public $name;
}
$f = fn(Holder $h): string => $h->name;
$g = function (Holder $h): string {
    return $h->name;
};
===expect===
NullableReturnStatement@6:30-6:38: Return type 'null|string' is not compatible with declared 'string'
NullableReturnStatement@8:4-8:20: Return type 'null|string' is not compatible with declared 'string'
