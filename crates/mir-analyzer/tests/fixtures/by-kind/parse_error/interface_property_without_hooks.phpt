===description===
Interface property without hooks
===file===
<?php
interface A {
    public string $value;
}
===expect===
ParseError@3:4-3:24: Parse error: Interfaces may only include hooked properties
