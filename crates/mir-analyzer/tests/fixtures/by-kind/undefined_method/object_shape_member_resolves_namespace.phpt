===description===
A class named in an `object{…}` shape resolves against the namespace and imports
like any other docblock name, in a `@var` and in a `@psalm-type` alias.
===file:Pool.php===
<?php
namespace Amp\Socket;

use Amp\Other\Clock;

interface Socket {
    public function isClosed(): bool;
}

/** @psalm-type Entry = object{object: Socket} */
final class Pool {
    /** @var list<Entry> */
    private array $entries = [];

    /** @var object{socket: Socket, clock: Clock} */
    private object $pair;

    public function __construct(object $pair) {
        $this->pair = $pair;
    }

    public function closed(): bool {
        foreach ($this->entries as $e) {
            if ($e->object->isClosed()) {
                return true;
            }
        }
        $this->pair->clock->tick();
        return $this->pair->socket->isClosed();
    }
}
===file:Clock.php===
<?php
namespace Amp\Other;

interface Clock {
    public function tick(): void;
}
