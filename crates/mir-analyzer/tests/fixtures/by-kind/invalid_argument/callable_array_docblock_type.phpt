===description===
`callable-array` is an array that is also a callable: `[object|class-string, existing-method]`.
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

final class Greeter
{
    public function greet(): void {}
}

/** @return callable-array */
function greeting()
{
    return [new Greeter(), 'greet'];
}

/** @param array<mixed> $value */
function takesArray(array $value): void {}

/** @param callable-array $value */
function takesCallableArray($value): void
{
    echo \count($value);
    $value();
}

takesArray(greeting());
takesCallableArray([new Greeter(), 'greet']);
takesCallableArray([Greeter::class, 'greet']);
takesCallableArray([new Greeter(), 'missing']);
//                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method App\Greeter::missing() does not exist
takesCallableArray([1, 2]);
//                 ^^^^^^ InvalidArgument: Argument $value of takesCallableArray() expects 'callable', got 'array{0: 1, 1: 2}'
takesCallableArray('strlen');
//                 ^^^^^^^^ InvalidArgument: Argument $value of takesCallableArray() expects 'array', got '"strlen"'
