===description===
Interface property without hooks
===file===
<?php
interface A {
    public string $value;
}
===expect===
ParseError@3:5-3:25: Parse error: Interfaces may only include hooked properties
