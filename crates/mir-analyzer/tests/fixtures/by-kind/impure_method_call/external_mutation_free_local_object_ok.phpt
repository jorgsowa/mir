===description===
Calling non-pure methods on a locally-created object inside a
@psalm-external-mutation-free method is allowed — only parameters are guarded.
===file===
<?php

class Builder {
    private array $parts = [];

    public function add(string $part): void {
        $this->parts[] = $part;
    }
}

class Factory {
    /** @psalm-external-mutation-free */
    public function make(string $part): Builder {
        $b = new Builder();
        $b->add($part);
        return $b;
    }
}
===expect===
