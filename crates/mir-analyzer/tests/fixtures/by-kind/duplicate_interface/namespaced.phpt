===description===
DuplicateInterface fires for a namespaced interface declared twice in the same file.
===file===
<?php
namespace App;

interface Repository
{
    public function find(int $id): mixed;
}

interface Repository
//<^ +3:1 DuplicateInterface: Interface App\Repository has already been defined
{
    public function findAll(): array;
}
===expect===
