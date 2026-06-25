===description===
P6(c): Backed enum missing a custom interface method emits UnimplementedInterfaceMethod (not confused by BackedEnum's synthesized methods).
===file===
<?php

interface Labelable {
    public function label(): string;
}

enum Status: string implements Labelable {
    case Active = 'active';
    case Inactive = 'inactive';
}
===expect===
UnimplementedInterfaceMethod@7:0-7:42: Class Status must implement Labelable::label() from interface
