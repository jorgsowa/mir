===description===
A declared class-string<Abstract> (parameter or call result) names a concrete subclass at runtime, so calling an abstract static method through it isn't flagged.
===file===
<?php
abstract class Uid {
    abstract public static function fromString(string $s): static;
}
abstract class UidType {
    /** @return class-string<Uid> */
    abstract protected function uidClass(): string;

    public function convert(string $value): Uid {
        return $this->uidClass()::fromString($value);
    }

    /** @param class-string<Uid> $cls */
    public function viaParam(string $cls, string $value): Uid {
        return $cls::fromString($value);
    }
}
