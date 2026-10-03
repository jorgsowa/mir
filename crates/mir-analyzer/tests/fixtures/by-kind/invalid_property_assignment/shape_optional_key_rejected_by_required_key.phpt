===description===
A shape whose key is merely optional can't satisfy a declared shape that
requires the same key present — an optional key may legally be absent at
runtime, unlike a genuinely required one.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

class Box {
    /** @var array{a: int} */
    public array $shape;
}

/** @param array{a?: int} $optionalShape */
function assign_optional_to_required(Box $b, array $optionalShape): void {
    $b->shape = $optionalShape;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $shape expects 'array{'a': int}', cannot assign 'array{'a'?: int}'
}

/** @param array{a: int} $requiredShape */
function assign_required_to_required(Box $b, array $requiredShape): void {
    $b->shape = $requiredShape;
}
===expect===
