===description===
DuplicateTrait fires when the same trait is declared twice.
===file===
<?php
trait Timestampable {
    public function getTimestamp(): int { return 0; }
}

trait Timestampable {
    public function updatedAt(): string { return ''; }
}
===expect===
DuplicateTrait@6:1-8:2: Trait Timestampable has already been defined
