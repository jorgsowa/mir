===description===
A bare `@psalm-assert T $x` on a static method narrows to the bound class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Vendor;

class Animal {}
class Dog extends Animal {}

class Guard {
    /**
     * @template T of object
     * @param mixed $value
     * @param class-string<T> $class
     * @psalm-assert T $value
     */
    public static function assertInstance($value, string $class): void {}
}

function test(mixed $value): void {
    Guard::assertInstance($value, Dog::class);
    /** @mir-check $value is Vendor\Dog */
    echo "ok";
}
===expect===
