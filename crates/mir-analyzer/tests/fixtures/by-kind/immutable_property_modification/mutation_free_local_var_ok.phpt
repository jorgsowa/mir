===description===
A @psalm-mutation-free method may assign to local variables — only $this
property writes are blocked.
===file===
<?php

class Formatter {
    public string $prefix = 'LOG';

    /** @psalm-mutation-free */
    public function format(string $message): string {
        $line = $this->prefix . ': ' . $message;
        $upper = strtoupper($line);
        return $upper;
    }
}
===expect===
