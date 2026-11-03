===description===
Of two declarations of a class, the one at its PSR-4 path wins, whatever the other's path
===file:src/App/Client.php===
<?php
namespace App;

final class Client {
    public function send(): void {}
}
===file:zz/fixtures/Client.php===
<?php
namespace App;

final class Client {}
===file:main.php===
<?php
(new App\Client())->send();
