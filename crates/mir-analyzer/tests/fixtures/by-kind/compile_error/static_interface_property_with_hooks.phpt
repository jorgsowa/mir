===description===
Static interface property with hooks
===file===
<?php
interface A {
    public static string $value { get; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Cannot declare hooks for static property [static-property-hooks]
}
