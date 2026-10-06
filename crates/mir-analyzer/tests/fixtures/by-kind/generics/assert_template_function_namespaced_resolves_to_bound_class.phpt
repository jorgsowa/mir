===description===
A bare `@psalm-assert T $x` on a namespaced function narrows to the class bound via class-string<T>, not `Vendor\T`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Vendor;

/**
 * @template T of object
 * @param mixed $value
 * @param class-string<T> $class
 * @psalm-assert T $value
 */
function assertInstance($value, string $class): void {}

class Animal {}
class Dog extends Animal {}

function test(mixed $value): void {
    assertInstance($value, Dog::class);
    /** @mir-check $value is Vendor\Dog */
    echo "ok";
}
