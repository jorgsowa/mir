===description===
A polyfill declaring a class of the enum's name behind a version check does not lend the enum its constructor, nor the class the enum's interfaces.
===file===
<?php
if (\PHP_VERSION_ID < 80100) {
    final class Mode {
        const A = 0;

        private function __construct() {}
    }
} else {
    enum Mode {
        case A;
    }
}
