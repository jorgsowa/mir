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
UnsupportedReferenceUsage@6:8-6:32: Reference assignment is not supported
