===description===
DuplicateTrait fires for a namespaced trait declared twice in the same file.
===file===
<?php
namespace App;

trait Timestampable
{
    public function createdAt(): int { return 0; }
}

trait Timestampable
{
    public function updatedAt(): int { return 0; }
}
===expect===
DuplicateTrait@9:0-12:1: Trait App\Timestampable has already been defined
