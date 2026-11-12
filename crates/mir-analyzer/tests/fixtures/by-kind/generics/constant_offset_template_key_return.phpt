===description===
`@return CONST[T]` with `T of key-of<CONST>` is the constant's value at the key the caller passed, or at the default key.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

const ID_TABLE = [
    'immutable' => 1,
    'mutable' => 'two',
];

function takesInt(int $value): void {}

function takesString(string $value): void {}

/**
 * @template T of key-of<ID_TABLE>
 * @param T $type
 * @return ID_TABLE[T]
 */
function lookUp(string $type = 'immutable'): int|string
{
    return ID_TABLE[$type];
}

takesInt(lookUp());
takesInt(lookUp('immutable'));
takesString(lookUp('mutable'));
takesString(lookUp('immutable'));
//          ^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesString() expects 'string', got '1'
takesInt(lookUp('mutable'));
//       ^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesInt() expects 'int', got '"two"'
