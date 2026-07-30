===description===
InaccessibleProperty does NOT fire when a subclass accesses a protected property declared on its parent.
===file===
<?php
class Base
{
    protected string $value = 'hidden';
}

class Child extends Base
{
    public function getValue(): string
    {
        return $this->value;
    }
}
===expect===
