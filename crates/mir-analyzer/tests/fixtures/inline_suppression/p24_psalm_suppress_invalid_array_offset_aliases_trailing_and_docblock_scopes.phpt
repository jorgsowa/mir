===description===
The alias works for a function-level docblock, and for `@psalm-suppress` combined
with other kinds in one directive.
===file===
<?php
/**
 * @param array{a: int} $v
 * @psalm-suppress InvalidArrayOffset, UndefinedClass
 */
function test(array $v): void {
    echo $v['missing'];
    new NoSuchClass();
}
===expect===
