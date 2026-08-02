===description===
A method implementing an interface (or overriding a parent/trait method)
keeps that contract's full parameter list even when its own body is a
no-op — the params can't be removed, so they aren't reported as unused.
===config===
suppress=MissingParamType
===file===
<?php
interface Logger {
    public function log($level, string $message, array $context = []): void;
}

class NullLogger implements Logger {
    public function log($level, string $message, array $context = []): void {
        // noop
    }
}
===expect===
