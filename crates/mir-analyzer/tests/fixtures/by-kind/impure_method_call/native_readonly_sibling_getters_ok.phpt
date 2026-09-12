===description===
Readonly-class methods are allowed in immutable contexts.
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
