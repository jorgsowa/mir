===description===
`template-type<Box<T>, Box, 'T'>` is the argument the object supplies for that template, so the call result is typed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

/** @template T */
final class Box {
    /** @param T $value */
    public function __construct(public mixed $value) {}
}

/**
 * @template T
 * @param Box<T> $box
 * @return template-type<Box<T>, Box, 'T'>
 */
function unwrap(Box $box): mixed {
    return $box->value;
}

function takesString(string $value): void {}

takesString(unwrap(new Box('text')));
takesString(unwrap(new Box(1)));
//          ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got 'int'
$number = unwrap(new Box(5));
/** @mir-check $number is int */
echo $number + 1;
