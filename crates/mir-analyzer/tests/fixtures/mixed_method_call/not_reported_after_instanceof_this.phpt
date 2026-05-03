===description===
not reported after instanceof this
===file===
<?php
abstract class A
{
    public function greet(): string { return ''; }

    public function equals(mixed $other): bool
    {
        if ($other instanceof $this) {
            $other->greet();
            return true;
        }
        return false;
    }
}
===expect===
===ignore===
TODO
