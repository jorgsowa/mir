===description===
`method_exists()` suppresses undefined methods for guarded `$this` callables.
===config===
suppress=MissingReturnType
===file===
<?php
class Widget {}

function register_shutdown(callable $cb): void {
    $cb();
}

class Consumer {
    public function maybe(): void {
        if (method_exists($this, 'gated')) {
            register_shutdown([$this, 'gated']);
        }
    }
}
===expect===
