===description===
mysqli_init() returns mysqli (not mysqli|false) on PHP >= 8.0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:Database.php===
<?php
class Database {
    private ?mysqli $connection = null;

    public function connect(): void {
        $this->connection = mysqli_init();
    }
}
