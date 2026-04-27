===file:Greetable.php===
<?php
namespace App;
trait Greetable {
    public function greet(): string { return 'hello'; }
}
===file:Farewell.php===
<?php
use App\Greetable;
trait Farewell {
    use Greetable;
    public function bye(): string { return 'bye'; }
}
class Host {
    use Farewell;
}
function test(): void {
    $h = new Host();
    $h->greet();
    $h->bye();
}
===expect===
