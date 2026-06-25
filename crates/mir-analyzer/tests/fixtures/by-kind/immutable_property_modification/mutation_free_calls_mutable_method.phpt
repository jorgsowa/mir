===description===
Calling a non-mutation-free method on $this from a @psalm-mutation-free method
emits ImpureMethodCall, even when the class itself is not @psalm-immutable.
===file===
<?php

class Logger {
    private array $entries = [];

    /** @psalm-mutation-free */
    public function flush(): void {
        $this->doClear();
    }

    private function doClear(): void {
        $this->entries = [];
    }
}
===expect===
ImpureMethodCall@8:8-8:24: Calling impure method doClear() in a pure or immutable context
