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
//      ^^^^^^^^^^^^^^^ MixedMethodCall: Method greet() called on mixed type
        return true;
    }
}
