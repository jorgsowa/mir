===description===
Interface missing attribute
===ignore===
TODO
===file===
<?php
interface I {
    public function f(): void;
}

interface I2 extends I {
    public function f(): void;
}

===expect===
