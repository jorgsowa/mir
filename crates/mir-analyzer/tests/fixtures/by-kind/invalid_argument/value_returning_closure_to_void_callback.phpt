===description===
Value-returning closures are valid for void callbacks.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
