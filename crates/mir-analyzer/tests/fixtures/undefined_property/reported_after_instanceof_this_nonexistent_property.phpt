===file===
<?php
abstract class A
{
    public string $value = '';

    public function test(mixed $other): void
    {
        if (! $other instanceof $this) {
            return;
        }
        echo $other->nonexistent;
    }
}
===expect===
UndefinedProperty: Property A::$nonexistent does not exist
