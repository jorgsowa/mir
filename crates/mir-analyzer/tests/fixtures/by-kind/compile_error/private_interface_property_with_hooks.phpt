===description===
Private interface property with hooks
===file===
<?php
interface A {
    private string $value { get; }
}
===expect===
CompileError@3:4-3:25: Compile error: Property in interface cannot be protected or private [invalid-interface-property-modifier]
