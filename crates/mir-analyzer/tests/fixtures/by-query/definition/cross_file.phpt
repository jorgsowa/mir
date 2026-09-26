===cursor===
definition
===file:src/Greeter.php===
<?php
namespace App;

final class Greeter {
    public function greet(): string { return 'hi'; }
}
===file:main.php===
<?php
use App\Greeter;

echo (new Greeter())->gr<CURSOR>eet();
===expect===
src/Greeter.php@5:4-5:52
