===description===
Valid PHP: passing a value-returning closure to a callback typed `Closure(): void`
is valid — PHP discards the returned value. mir checks the closure's return type
against `void` and reports the non-void return as an invalid argument.
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
