===description===
@var generic argument is FQN-resolved through use statement (regression guard: was stored as short name causing type-check FPs)
===config===
suppress=UnusedParam,UnusedProperty,UnusedVariable
===file:Lib/Container.php===
<?php
namespace Lib;

/** @template T */
class Container {}
===file:App/Context.php===
<?php
namespace App;

class Context {}
===file:App/Test.php===
<?php
namespace App;

use Lib\Container;

class MyTest {
    public function test(): void {
        /** @var Container<Context> $c */
        $c = new Container();
        /** @mir-check $c is Lib\Container<App\Context> */
        echo "ok";
    }
}
===expect===
