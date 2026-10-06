===description===
No warning when an interface declares __toString and a concrete implementation is passed to a string param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Printable {
    public function __toString();
}

class Report implements Printable {
    public function __toString() { return 'report'; }
}

function log(string $message): void {}

log(new Report());
