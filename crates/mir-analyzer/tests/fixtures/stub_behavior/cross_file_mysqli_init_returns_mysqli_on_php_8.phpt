===description===
mysqli_init() returns mysqli (not mysqli|false) on PHP >= 8.0
===config===
php_version=8.0
===file:Database.php===
<?php
class Database {
    private ?mysqli $connection = null;

    public function connect(): void {
        $this->connection = mysqli_init();
    }
}
===expect===
