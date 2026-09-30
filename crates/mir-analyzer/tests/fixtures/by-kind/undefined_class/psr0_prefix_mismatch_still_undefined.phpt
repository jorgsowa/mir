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
//                         ^^^^^^^^^^^^ UndefinedClass: Class Other\Thing does not exist
    }
}
===expect===
