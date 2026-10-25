===description===
Protected interface property with hooks
===file===
<?php
interface A {
    protected string $value { get; }
//  ^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Property in interface cannot be protected or private [invalid-interface-property-modifier]
}
