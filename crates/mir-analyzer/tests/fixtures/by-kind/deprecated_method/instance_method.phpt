===description===
DeprecatedMethod fires when calling a deprecated instance method.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Logger {
    /** @deprecated use log() instead */
    public function write(string $msg): void {}
}

function test(Logger $l): void {
    $l->write('hello');
//  ^^^^^^^^^^^^^^^^^^ DeprecatedMethod: Method Logger::write() is deprecated: use log() instead
}
===expect===
