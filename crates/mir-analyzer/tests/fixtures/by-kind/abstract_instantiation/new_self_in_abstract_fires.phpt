===description===
new self() inside an abstract class fires AbstractInstantiation — use new static() for LSB instead.
===file===
<?php
abstract class Base {
    public function create(): void {
        new self();
    }
}
===expect===
AbstractInstantiation@4:12-4:16: Cannot instantiate abstract class Base
