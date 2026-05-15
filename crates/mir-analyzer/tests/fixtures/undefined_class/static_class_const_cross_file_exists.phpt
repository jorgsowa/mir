===description===
Foo::class cross file exists — no UndefinedClass when class is defined in another file
===file:Router.php===
<?php
namespace App;
class Router {}
===file:Container.php===
<?php
use App\Router;
function getRouterClass(): string {
    return Router::class;
}
===expect===
