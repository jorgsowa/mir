===description===
ref assignment not reported
===file===
<?php
final class MyClass {
    private \stdClass $config;

    public function __construct(\stdClass $config) {
        $this->config = &$config;
    }
}
===expect===
