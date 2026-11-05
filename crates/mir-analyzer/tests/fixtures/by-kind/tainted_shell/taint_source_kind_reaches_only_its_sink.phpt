===file===
<?php
/** @psalm-taint-source shell */
function source(): string { return ''; }

class Input {
    /** @taint-source shell */
    public function get(): string { return ''; }
    /** @psalm-taint-source shell */
    public static function read(): string { return ''; }
}

echo source();
echo (new Input())->get();
echo Input::read();
exec(source());
//<^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
exec((new Input())->get());
//<^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
exec(Input::read());
//<^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
