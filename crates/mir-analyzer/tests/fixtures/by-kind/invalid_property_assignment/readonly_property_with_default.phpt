===description===
A native readonly property may not carry a default value — a PHP fatal.
===file===
<?php
class A {
    public readonly string $s = "a";
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReadonlyPropertyDeclaration: Readonly property A::$s cannot have a default value
}
