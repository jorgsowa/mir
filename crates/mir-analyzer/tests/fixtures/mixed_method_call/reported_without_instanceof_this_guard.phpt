===description===
reported without instanceof this guard
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
MixedMethodCall@8:8: Method greet() called on mixed type
===ignore===
TODO
