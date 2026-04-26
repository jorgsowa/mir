===file:Domain/NotFoundException.php===
<?php
namespace Domain;

class NotFoundException extends \RuntimeException {}
===file:App/Repository.php===
<?php
namespace App;

use Domain\NotFoundException;

class Repository {
    public function find(int $id): object {
        throw new NotFoundException("id $id not found");
    }
}
===file:Main.php===
<?php
use App\Repository;
use Domain\NotFoundException;

$repo = new Repository();
try {
    $repo->find(42);
} catch (NotFoundException $e) {
    echo $e->getMessage();
}
===expect===
