===description===
From PHP 8.6 a readonly property's default value initializes it, so a later write fails; __clone may reinitialize it.
===config===
<mir>
  <phpVersion>8.6</phpVersion>
</mir>
===file===
<?php
class A {
    public readonly int $x = 1;
    public readonly int $y;

    public function __construct() {
        $this->x = 2;
//      ^^^^^^^^^^^^ ReadonlyPropertyAlreadyInitialized: Cannot modify readonly property A::$x — already initialized
        $this->y = 2;
    }

    public function __clone() {
        $this->x = 3;
    }
}
