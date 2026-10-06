===description===
@var T $this->prop refines the property and is not an unused variable
===file===
<?php
class Foo {
    public mixed $prop = null;
    public function m(): void {
        /** @var int $this->prop */
        $this->prop = 1;
        /** @mir-check $this->prop is int */
        echo $this->prop;
    }
}
