===description===
A final class named only in a function's `@return` docblock tag (no native
return type naming it) must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Foo {}

/**
 * @return ?Foo
 */
function makeFoo(): mixed {
    return null;
}

makeFoo();
