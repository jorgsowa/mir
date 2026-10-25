===description===
Private interface property with hooks
===file===
<?php
interface A {
    private string $value { get; }
//  ^^^^^^^^^^^^^^^^^^^^^ CompileError: Property in interface cannot be protected or private [invalid-interface-property-modifier]
}
