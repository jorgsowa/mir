===description===
Interface property without hooks
===file===
<?php
interface A {
    public string $value;
//  ^^^^^^^^^^^^^^^^^^^^ CompileError: Interfaces may only include hooked properties [interface-property-without-hooks]
}
