===description===
Class missing attribute
===ignore===
TODO
===file===
<?php
class C {
    public function f(): void {}
}

class C2 extends C {
    public function f(): void {}
}

===expect===
