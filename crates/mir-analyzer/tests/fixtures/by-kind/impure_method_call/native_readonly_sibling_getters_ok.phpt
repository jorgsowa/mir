===description===
Unannotated instance methods of a PHP 8.2+ native readonly class are mutation-free
by construction: the language forbids property reassignment after construction, so
a sibling getter called from another method must not be flagged as ImpureMethodCall.
===file===
<?php

declare(strict_types=1);

final readonly class Runtime {
    public function __construct(
        private string $name,
        private int $versionId,
    ) {}

    public function name(): string {
        return $this->name;
    }

    public function versionId(): int {
        return $this->versionId;
    }

    public function asString(): string {
        return 'X' . $this->name() . '#' . (string) $this->versionId();
    }
}
===expect===
