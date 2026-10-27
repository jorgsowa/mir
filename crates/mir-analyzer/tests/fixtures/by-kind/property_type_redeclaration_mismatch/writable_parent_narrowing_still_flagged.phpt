===description===
A plain (writable) parent property stays invariant: narrowing the type is still an error.
===file===
<?php
abstract class Id {
    public int|string $value = 1;
}

final class NarrowId extends Id {
    public string $value = 'a';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeRedeclarationMismatch: Type of NarrowId::$value must be string|int (as in class Id)
}
