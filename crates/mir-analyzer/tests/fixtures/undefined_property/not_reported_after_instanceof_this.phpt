===file===
<?php
abstract class A
{
    public string $value = '';

    public function equals(mixed $other): bool
    {
        if (! $other instanceof $this) {
            return false;
        }
        return $this->value === $other->value;
    }
}
===expect===
