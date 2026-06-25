===description===
The constructor of a @psalm-immutable class is not subject to immutability
enforcement — calling a non-mutation-free method on $this from __construct
does not emit ImpureMethodCall.
===file===
<?php

/** @psalm-immutable */
class Config {
    public function __construct(
        public string $dsn,
        public int $timeout,
    ) {
        $this->validate();
    }

    private function validate(): void {
        if (strlen($this->dsn) === 0) {
            throw new \InvalidArgumentException('DSN cannot be empty');
        }
    }
}
===expect===
