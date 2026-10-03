===description===
An imported unsealed shape (`...array<string, mixed>`) accepts extra keys but
still requires its declared ones.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:logger.php===
<?php
namespace Log;

/** @psalm-type Context = array{exception?: \Throwable, ...array<string, mixed>} */
class Logger {
    /** @param Context $context */
    public function log(array $context): void {}
}
===file:use.php===
<?php
namespace App;

use Log\Logger;

/** @psalm-import-type Context from Logger */
class Job {
    /** @param Context $c */
    public function ctx(array $c, Logger $l): void {
        $l->log($c);
        $l->log(['exception' => new \Exception(), 'job' => 1]);
        $l->log(['job' => 1]);
    }
}
===expect===
