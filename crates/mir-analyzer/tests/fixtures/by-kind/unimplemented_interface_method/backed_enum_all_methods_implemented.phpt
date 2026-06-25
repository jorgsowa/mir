===description===
P6(c): Backed enum that implements all required custom interface methods is not reported. The synthesized from()/tryFrom()/cases() methods are not checked.
===file===
<?php

interface Labelable {
    public function label(): string;
}

enum Status: string implements Labelable {
    case Active = 'active';
    case Inactive = 'inactive';

    public function label(): string
    {
        return ucfirst($this->value);
    }
}
===expect===
