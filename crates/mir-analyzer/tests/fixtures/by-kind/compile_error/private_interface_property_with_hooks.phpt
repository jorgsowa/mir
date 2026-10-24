===description===
Private interface property with hooks
===file===
<?php
interface A {
    private string $value { get; }
//  ^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Property in interface cannot be protected or private [invalid-interface-property-modifier]
}
