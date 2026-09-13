===description===
Value-returning closures are valid for void callbacks.
===ignore===
===config===
php_version=8.4
===file===
<?php
final class Invoker
{
       /** @param Closure(): void $callback */
    public function run(Closure $callback): void
      {
           $callback();
       }

    public function go(): void
      {
           $this->run(fn() => 1);
          $this->run(static function (): int {
            return 2;
          });
       }
}
===expect===
