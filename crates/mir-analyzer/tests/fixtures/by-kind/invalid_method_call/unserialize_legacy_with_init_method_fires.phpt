===description===
$this->__construct() in a helper called from the legacy Serializable::unserialize() still fires (exemption is method-direct only)
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function __construct() {}
    public function init(): void {
        $this->__construct();
    }
    public function unserialize(string $data): void {
        $this->init();
    }
}
===expect===
DirectConstructorCall@5:8-5:28: Cannot call constructor of A directly
