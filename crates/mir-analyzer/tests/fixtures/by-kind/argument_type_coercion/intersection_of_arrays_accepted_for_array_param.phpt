===description===
An intersection of array types satisfies a plain array parameter.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Logger {
    /** @param array<string, mixed>&array{id: int} $context */
    public function log(array $context): void {
        $this->write($context);
    }
    public function write(array $context): void {}
}
===expect===
