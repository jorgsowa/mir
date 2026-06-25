===description===
Calling a @psalm-mutation-free method on a parameter inside a
@psalm-external-mutation-free method is allowed — mutation-free guarantees no
mutation of the receiver ($this inside the callee).
===file===
<?php

class Formatter {
    private string $prefix = 'X';

    /** @psalm-mutation-free */
    public function format(string $s): string {
        return $this->prefix . $s;
    }
}

class Processor {
    /** @psalm-external-mutation-free */
    public function run(Formatter $fmt, string $input): string {
        return $fmt->format($input);
    }
}
===expect===
