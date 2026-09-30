===description===
Each parameter that has a non-pure method called on it fires a separate
ImpureMethodCall in a @psalm-external-mutation-free method.
===file===
<?php

class Writer {
    public function write(string $s): void { error_log($s); }
}

class Reader {
    public function read(): string { return ''; }
}

class Pipe {
    /** @psalm-external-mutation-free */
    public function transfer(Reader $r, Writer $w): void {
        $w->write($r->read());
//      ^^^^^^^^^^^^^^^^^^^^^ ImpureMethodCall: Calling impure method write() in a pure or immutable context
//                ^^^^^^^^^^ ImpureMethodCall: Calling impure method read() in a pure or immutable context
    }
}
===expect===
