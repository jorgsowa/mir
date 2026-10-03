===description===
D4: an arrow function's `=> expr` is exactly one implicit `return expr;` — a
nullable property read against a non-nullable declared return type must flag
the same way an equivalent `function(){...}` closure already does.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Holder {
    /** @var null|string */
    public $name;
}
$f = fn(Holder $h): string => $h->name;
//                            ^^^^^^^^ NullableReturnStatement: Return type 'null|string' is not compatible with declared 'string'
$g = function (Holder $h): string {
    return $h->name;
//  ^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'null|string' is not compatible with declared 'string'
};
===expect===
