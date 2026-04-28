===file===
<?php
class Base {
    public function getInstance(): static { return $this; }
}
class Child extends Base {
    public function getInstance(): static { return $this; }
}
===expect===
