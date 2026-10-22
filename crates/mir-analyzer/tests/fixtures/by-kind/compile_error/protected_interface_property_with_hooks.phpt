===description===
Protected interface property with hooks
===file===
<?php
interface A {
    protected string $value { get; }
}
===expect===
CompileError@3:4-3:27: Compile error: Property in interface cannot be protected or private [invalid-interface-property-modifier]
