===description===
P6(c): Enum that implements interface A (which extends interface B) must also implement B's methods.
===file===
<?php

interface Printable {
    public function print(): void;
}

interface PrettyPrintable extends Printable {
    public function prettyPrint(): void;
}

enum Color implements PrettyPrintable {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Color must implement Printable::print() from interface
    case Red;
    case Green;

    public function prettyPrint(): void
    {
        echo $this->name;
    }
    // Missing print() from Printable (via PrettyPrintable)
}
