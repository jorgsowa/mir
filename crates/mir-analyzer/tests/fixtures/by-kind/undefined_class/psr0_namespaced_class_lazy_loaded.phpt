===description===
A class reachable only via a `psr-0` autoload entry (no PSR-4 mapping) is
lazily resolvable by FQCN, so a type hint referencing it does not false-
positive as UndefinedClass. PSR-0 keeps the namespace directory structure in
the path (unlike PSR-4, which strips the matched prefix segment).
===file:composer.json===
{"autoload":{"psr-0":{"Mailer\\":"src/"}}}
===file:src/Mailer/Message.php===
<?php
namespace Mailer;
class Message {
    public function body(): string { return ''; }
}
===file:Handler.php===
<?php
namespace App;
use Mailer\Message;
class Handler {
    public function handle(Message $m): string {
        return $m->body();
    }
}
===expect===
