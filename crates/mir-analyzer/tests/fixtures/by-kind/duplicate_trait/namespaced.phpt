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
//<^ +3:1 DuplicateTrait: Trait App\Timestampable has already been defined
{
    public function updatedAt(): int { return 0; }
}
===expect===
