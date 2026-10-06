===description===
A template nested in an asserted type (`list<T>`) is substituted too.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Vendor;

class Dog {}

/**
 * @template T of object
 * @param mixed $value
 * @param class-string<T> $class
 * @psalm-assert list<T> $value
 */
function assertListOf($value, string $class): void {}

function test(mixed $value): void {
    assertListOf($value, Dog::class);
    /** @mir-check $value is list<Vendor\Dog> */
    echo "ok";
}
