===description===
A psr-0 autoload entry must not over-match: a class outside the mapped
prefix's namespace is still genuinely undefined.
===config===
suppress=UnusedParam
===file:composer.json===
{"autoload":{"psr-0":{"Mailer\\":"src/"}}}
===file:Handler.php===
<?php
namespace App;
class Handler {
    public function handle(\Other\Thing $t): void {
    }
}
===expect===
Handler.php: UndefinedClass@4:27-4:39: Class Other\Thing does not exist
