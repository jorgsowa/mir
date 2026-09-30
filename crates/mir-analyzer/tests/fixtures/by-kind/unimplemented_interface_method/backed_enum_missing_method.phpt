===description===
P6(c): Backed enum missing a custom interface method emits UnimplementedInterfaceMethod (not confused by BackedEnum's synthesized methods).
===file===
<?php

interface Labelable {
    public function label(): string;
}

enum Status: string implements Labelable {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Status must implement Labelable::label() from interface
    case Active = 'active';
    case Inactive = 'inactive';
}
===expect===
