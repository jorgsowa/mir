===description===
`callable-object` is an object with `__invoke()`; a `Closure` qualifies.
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

final class Invokable
{
    public function __invoke(): void {}
}

final class NotInvokable {}

/** @return callable-object */
function invokable()
{
    return new Invokable();
}

/** @return callable-object */
function notInvokable()
{
    return new NotInvokable();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'App\NotInvokable' is not compatible with declared 'object&callable'
}

function takesObject(object $value): void {}

/** @param callable-object $value */
function takesCallableObject($value): void
{
    $value();
}

takesObject(invokable());
takesCallableObject(new Invokable());
takesCallableObject(static fn (): int => 1);
takesCallableObject(new NotInvokable());
//                  ^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesCallableObject() expects 'callable', got 'App\NotInvokable'
takesCallableObject('strlen');
//                  ^^^^^^^^ InvalidArgument: Argument $value of takesCallableObject() expects 'object', got '"strlen"'
