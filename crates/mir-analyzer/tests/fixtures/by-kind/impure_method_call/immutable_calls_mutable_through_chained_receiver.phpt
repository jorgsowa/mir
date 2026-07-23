===description===
A method call through a chained property receiver ($this->logger->write(),
the delegation pattern) escaped @psalm-immutable checks entirely -- the
check only matched when call.object was LITERALLY `$this`, never when
`$this` was reached through an intermediate property in the chain. This is
the call-side sibling of the already-fixed write-side chain walking.
===config===
suppress=MissingConstructor
===file===
<?php
class Logger {
    public function write(): void {
    }
}

/** @psalm-immutable */
class Service {
    public Logger $logger;

    public function run(): void {
        $this->logger->write();
    }
}
===expect===
ImpureMethodCall@12:8-12:30: Calling impure method write() in a pure or immutable context
