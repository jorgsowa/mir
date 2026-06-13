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
UnsupportedReferenceUsage@6:9-6:33: Reference assignment is not supported
