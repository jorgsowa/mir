===description===
DuplicateTrait fires when the same trait is declared twice.
===file===
<?php
trait Timestampable {
    public function getTimestamp(): int { return 0; }
}

trait Timestampable {
//<^ +2:1 DuplicateTrait: Trait Timestampable has already been defined
    public function updatedAt(): string { return ''; }
}
