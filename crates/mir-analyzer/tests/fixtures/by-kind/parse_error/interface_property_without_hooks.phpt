===description===
Interface property without hooks
===file===
<?php
interface A {
    public string $value;
//  ^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: Interfaces may only include hooked properties
}
