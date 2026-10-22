===description===
Interface property without hooks
===file===
<?php
interface A {
    public string $value;
//  ^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Interfaces may only include hooked properties [interface-property-without-hooks]
}
