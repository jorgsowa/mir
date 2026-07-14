===description===
A native readonly property may not carry a default value — a PHP fatal.
===file===
<?php
class A {
    public readonly string $s = "a";
}
===expect===
InvalidReadonlyPropertyDeclaration@3:4-3:35: Readonly property A::$s cannot have a default value
