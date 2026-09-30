===description===
new self() inside an abstract class fires AbstractInstantiation — use new static() for LSB instead.
===file===
<?php
abstract class Base {
    public function create(): void {
        new self();
//          ^^^^ AbstractInstantiation: Cannot instantiate abstract class Base
    }
}
===expect===
