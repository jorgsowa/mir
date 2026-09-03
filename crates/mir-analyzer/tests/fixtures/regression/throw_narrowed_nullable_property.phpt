===description===
FP-N (broader): `throw $this->exception` guarded by `if ($this->exception !== null)` —
the null check narrows the Throwable|null property to Throwable, so InvalidThrow
must not be emitted.
===config===
php_version=8.2
===file===
<?php

class Handler {
    private ?\RuntimeException $exception = null;

    public function process(): void {
        if ($this->exception !== null) {
            throw $this->exception;
        }
    }

    public function processChained(): void {
        if ($this->exception === null) {
            return;
        }
        throw $this->exception;
    }
}
===expect===
