===description===
`callable-array`, `callable-object`, `static-closure` and `static-pure-closure` accept callables of their kind and reject other values.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

final class Greeter
{
    public function greet(): void {}

    public function __invoke(): void {}
}

/** @param callable-array $value */
function acceptsCallableArray($value): void {}

/** @param callable-object $value */
function acceptsCallableObject($value): void {}

/** @param static-closure $value */
function acceptsStaticClosure($value): void {}

/** @param static-pure-closure $value */
function acceptsStaticPureClosure($value): void {}

acceptsCallableArray([new Greeter(), 'greet']);
acceptsCallableArray('greet');
//                   ^^^^^^^ InvalidArgument: Argument $value of acceptsCallableArray() expects 'array', got '"greet"'

acceptsCallableObject(new Greeter());
acceptsCallableObject(static fn (): int => 1);
acceptsCallableObject('greet');
//                    ^^^^^^^ InvalidArgument: Argument $value of acceptsCallableObject() expects 'object', got '"greet"'

acceptsStaticClosure(static fn (): int => 1);
acceptsStaticClosure(new Greeter());
//                   ^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsStaticClosure() expects 'Closure(mixed): mixed', got 'App\Greeter'

acceptsStaticPureClosure(static fn (int $value): int => $value + 1);
acceptsStaticPureClosure([new Greeter(), 'greet']);
//                       ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of acceptsStaticPureClosure() expects 'pure-Closure(mixed): mixed', got 'array{0: App\Greeter, 1: "greet"}'
