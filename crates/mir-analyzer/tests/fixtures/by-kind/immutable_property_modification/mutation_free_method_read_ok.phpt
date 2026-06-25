===description===
A @psalm-mutation-free method may freely read $this properties — no error.
===file===
<?php

class Config {
    public string $name = 'default';
    public int $timeout = 30;

    /** @psalm-mutation-free */
    public function getName(): string {
        return $this->name;
    }

    /** @psalm-mutation-free */
    public function getTimeout(): int {
        return $this->timeout;
    }
}
===expect===
