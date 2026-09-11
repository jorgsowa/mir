===description===
method_exists() guard suppresses UndefinedMethod for a [$this, 'method'] array callable
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
