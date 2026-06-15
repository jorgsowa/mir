===description===
reported after instanceof this nonexistent property
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
UndefinedProperty@11:21-11:32: Property A::$nonexistent does not exist
