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
{
    public function findAll(): array;
}
===expect===
DuplicateInterface@9:0-12:1: Interface App\Repository has already been defined
