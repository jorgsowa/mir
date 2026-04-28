===file===
<?php
abstract class A
{
    public function greet(): string { return ''; }

    public function equals(mixed $other): bool
    {
        $other->greet();
        return true;
    }
}
===expect===
MixedMethodCall: Method greet() called on mixed type
